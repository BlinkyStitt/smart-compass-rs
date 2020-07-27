#![no_std]

pub mod battery;
// pub mod compass;
// pub mod config;
pub mod lights;
pub mod location;
pub mod network;
pub mod periodic;
pub mod storage;

pub use accelerometer;

pub const MAX_PEERS: usize = 5;

// TODO: use rtic resources instead
pub static mut ELAPSED_MS: u32 = 0;
