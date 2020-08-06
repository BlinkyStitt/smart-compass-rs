use super::{Pattern, ANGLES, RGB8};
use crate::arduino::map;
use crate::lights::focalintent::{triangle88, Accum88};
use core::cmp::Ordering;
use smart_leds::colors::{BLACK, YELLOW};

pub struct PacMan {
    bites_per_minute: Accum88,
    open_mouth_angle: u8,
}

impl PacMan {
    pub fn new() -> Self {
        Self {
            // TODO: accum88 has a max bpm that is too low. maybe better to use Accum124
            bites_per_minute: (250u8).into(),
            // pacman's mouth is 55 degrees. 55 / 360 * 256 = 39
            // TODO: but that look too small
            open_mouth_angle: 39,
        }
    }
}

impl Pattern for PacMan {
    fn buffer(&mut self, now: u32, leds: &mut [RGB8]) {
        // draw mouth
        // TODO: i don't think triangle wave looks right. mouth stays
        let mouth_angle = map(
            triangle88(self.bites_per_minute, now),
            -1.0,
            1.0,
            0.0,
            self.open_mouth_angle as f32,
        ) as u8;

        // TODO: start_angle needs to change
        angle_fill(
            leds,
            255 - (mouth_angle / 2),
            mouth_angle,
            &BLACK,
            Some(&YELLOW),
        );

        // TODO: draw eye
    }
}

fn angle_fill(
    leds: &mut [RGB8],
    start_angle: u8,
    end_angle: u8,
    inside_color: &RGB8,
    outside_color: Option<&RGB8>,
) {
    // const SMALLEST_ANGLE: u8 = 40;

    // TODO: don't draw thee smallest angle

    let wrap_around = match start_angle.cmp(&end_angle) {
        Ordering::Equal => None,
        Ordering::Greater => Some(true),
        Ordering::Less => Some(false),
    };

    for (i, led) in leds.iter_mut().enumerate() {
        let angle = ANGLES[i];

        let is_inside = if i == 0 {
            // TODO: how should we handle the first LED?
            false
        } else {
            match (wrap_around, angle.cmp(&start_angle), angle.cmp(&end_angle)) {
                (None, _, _) => false,
                (_, Ordering::Equal, _) => true,
                (_, _, Ordering::Equal) => true,
                (Some(false), Ordering::Greater, Ordering::Less) => true,
                (Some(true), Ordering::Less, Ordering::Less) => true,
                (Some(true), Ordering::Greater, Ordering::Greater) => true,
                _ => false,
            }
        };

        if is_inside {
            *led = *inside_color;
        } else if let Some(outside_color) = outside_color {
            *led = *outside_color;
        }
    }
}
