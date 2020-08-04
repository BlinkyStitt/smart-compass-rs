use super::Pattern;
use super::COORDS_X;
use super::RGB8;
use crate::lights::focalintent::fade_to_black_by;
use smart_leds::colors::RED;

pub struct TestMap {
    pixel_map: &'static [u8],
    active: usize,
}

impl TestMap {
    pub fn new() -> Self {
        Self {
            pixel_map: &COORDS_X,
            active: 0,
        }
    }
}

impl Pattern for TestMap {
    fn buffer(&mut self, _now: u32, leds: &mut [RGB8]) {
        fade_to_black_by(leds, 64);

        // let i = self.pixel_map[self.active] as usize;
        let i = self.active;

        leds[i] = RED;

        if self.active < leds.len() {
            self.active += 1;
        } else {
            self.active = 0;
        }
    }
}
