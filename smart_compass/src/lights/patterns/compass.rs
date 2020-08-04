use super::RGB8;
use crate::network::PeerLocations;

pub struct Compass {}

impl Compass {
    pub fn buffer(&mut self, now: u32, leds: &mut [RGB8], peer_locations: &PeerLocations) {
        todo!();
    }
}
