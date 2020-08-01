#![no_std]

pub mod battery;
// pub mod compass;
// pub mod config;
pub mod lights;
pub mod location;
pub mod network;
pub mod storage;
pub mod timers;

pub use accelerometer;

// TODO: i'd like this to be configurable at runtime,
pub const MAX_PEERS: usize = 5;
pub const NUM_LEDS: usize = 256;
