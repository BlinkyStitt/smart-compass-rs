//! every dot is moving in a straight line
use super::Pattern;
use super::RGB8;
use smart_leds::colors::RED;
use derive_more::Constructor;
use super::compass::bearing_and_distance_to_id;
use super::super::focalintent::{Accum88, beatsin};
use crate::arduino::map;

#[derive(Constructor)]
pub struct Lines {
    ms_per_angle: u32,
}

impl Pattern for Lines {
    fn buffer(&mut self, now: u32, leds: &mut [RGB8]) {
        // TODO: check for off-by-one errors
        // get a number from 0 to u16::MAX for the distance
        let distance = beatsin(Accum88::from(120u8), 0, u16::MAX, now, 0) as f32;

        // get a number from -255 to 255 for the distance
        let distance = map(distance, 0.0, u16::MAX as f32, -255.0, 255.0);

        // get a number from 0 to 360 for the angle
        let angle = 0.0;

        let id = bearing_and_distance_to_id(angle, distance, 255.0);

        let color = RED;

        leds[id] = color;
    }
}
