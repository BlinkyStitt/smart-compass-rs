use super::{Pattern, ANGLES, RGB8};
use crate::arduino::map;
use crate::lights::focalintent::{trianglewave, Accum88};
use core::cmp::Ordering;
use smart_leds::colors::{BLACK, YELLOW};
use core::f32::consts::{FRAC_2_PI, PI};

pub struct PacMan {
    bites_per_minute: Accum88,
    open_mouth_angle: u8,
}

impl PacMan {
    pub fn new() -> Self {
        Self {
            bites_per_minute: Accum88::from(140u8),
            // pacman's mouth is 55 degrees. 55 / 360 * 256 = 39
            open_mouth_angle: 39,
        }
    }
}

impl Pattern for PacMan {
    fn buffer(&mut self, now: u32, leds: &mut [RGB8]) {
        // draw mouth
        // TODO: i don't think triangle wave looks right. mouth stays
        // TODO: have a "dying" state that makes the mouth fill the whole amount
        let mouth_width = map(
            trianglewave(self.bites_per_minute, now),
            -1.0 * FRAC_2_PI,
            FRAC_2_PI,
            0.0,
            self.open_mouth_angle as f32,
        ) as u8;

        // TODO: start_angle needs to change
        // TODO: have a "direction" that changes angle from 0 to 128
        angle_fill_centered(leds, 0, mouth_width, &BLACK, Some(&YELLOW));

        // TODO: draw eye
    }
}

fn angle_fill_centered(
    leds: &mut [RGB8],
    angle: u8,
    width: u8,
    inside_color: &RGB8,
    outside_color: Option<&RGB8>,
) {
    let width_1 = width / 2;
    let width_2 = width - width_1;

    let (start_angle, _) = angle.overflowing_sub(width_1);
    let (end_angle, _) = angle.overflowing_add(width_2);

    angle_fill(leds, start_angle, end_angle, inside_color, outside_color)
}

fn angle_fill(
    leds: &mut [RGB8],
    start_angle: u8,
    end_angle: u8,
    inside_color: &RGB8,
    outside_color: Option<&RGB8>,
) {
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
