use super::clock::antialias_pixel_ar;
use super::Pattern;
use super::RGB8;
use crate::arduino::map;
use crate::lights::focalintent::{beat88, beatsin88, Accum88};
use smart_leds::colors::{BLACK, YELLOW};

pub struct PacMan {
    bites_per_minute: Accum88,
    mouth_open_degrees: u8,
}

impl PacMan {
    pub fn new() -> Self {
        Self {
            bites_per_minute: 60u8.into(),
            // pacman's mouth is 55 degrees. 55 / 360 * 256
            mouth_open_degrees: 39,
        }
    }
}

impl Pattern for PacMan {
    fn buffer(&mut self, now: u32, leds: &mut [RGB8]) {
        for led in leds.iter_mut() {
            *led = YELLOW;
        }

        // TODO: draw eye

        // draw mouth
        // TODO: beat88 is a sawtooth wave. we want a different wave
        /*
        let mouth_angle = map(
            beat88(self.bites_per_minute, now),
            u16::MIN,
            u16::MAX,
            0,
            self.mouth_open_degrees as u16,
        ) as u8;
        */

        /*
        let mouth_angle = map(
            triangle88(
                self.bites_per_minute,
                now
            ),
            i16::MIN,
            i16::MAX,
            0,
            self.mouth_open_degrees as i16,
        ) as u8;
        */

        antialias_pixel_ar(leds, 200, 20, 0, 255, BLACK);
    }
}
