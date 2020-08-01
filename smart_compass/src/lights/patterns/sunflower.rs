use super::{hsv2rgb, Hsv, Pattern, PHYSICAL_TO_FIBONACCI, RGB8};
use crate::timers::ElapsedMs;

enum Direction {
    In,
    Out,
}

pub struct Sunflower {
    direction: Direction,
    pixel_map: &'static [u8],
}

impl Sunflower {
    pub fn new() -> Self {
        Self {
            direction: Direction::Out,
            pixel_map: &PHYSICAL_TO_FIBONACCI,
        }
    }
}

impl Pattern for Sunflower {
    /// TODO: fastled did something special for rainbows. do the same here
    fn draw(&mut self, now: u32, leds: &mut [RGB8]) {
        // divide to slow down the animation.
        // TODO: or we could advance by 1 per frame
        let now = now / 30;

        for (i, led) in leds.iter_mut().enumerate() {
            let hue = match self.direction {
                Direction::In => now + (self.pixel_map[i] as u32 * 3) / 7,
                Direction::Out => now - (self.pixel_map[i] as u32 * 3) / 7,
            } as u8;

            let new_color = hsv2rgb(Hsv {
                hue,
                sat: 240,
                val: 255,
            });

            // TODO: is there a better way to do this?
            led.r = new_color.r;
            led.g = new_color.g;
            led.b = new_color.b;
        }
    }
}
