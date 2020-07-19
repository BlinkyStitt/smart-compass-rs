// TODO: the adafruit_gps crate requires std::io! looks like we need to roll our own
// use adafruit_gps::gps::{Gps, GpsSentence};
// use adafruit_gps::send_pmtk::NmeaOutput;
use crate::UART0;

pub struct Gps {
    uart: UART0
}

impl Gps {
    pub fn new(uart: UART0) -> Self {
        Gps { uart }
    }

    pub fn read(&mut self) {
        todo!()
    }
}

// TODO: the old code read the gps data on a timer. do we want that still?
// https://github.com/atsamd-rs/atsamd/blob/master/boards/feather_m0/examples/timers.rs

/*
// TODO: serde serialize/deserialize on this
struct SmartCompassLocationMessage {
    bytes network_hash = 1 [(nanopb).max_size = 16, (nanopb).fixed_length = true];
    bytes message_hash = 2 [(nanopb).max_size = 16, (nanopb).fixed_length = true];
    uint32 tx_peer_id = 3;

    uint32 tx_time = 4;
    uint32 tx_ms = 5;

    uint32 peer_id = 6;
    uint32 last_updated_at = 7;
    uint32 hue = 8; // todo: fixed_length and max_size = 8 bits?
    uint32 saturation = 9; // todo: fixed_length and max_size = 8 bits?
    int32 latitude = 10;
    int32 longitude = 11;

    // todo: if there is a mismatch between peers, we need to re-broadcast old pins
    // todo: this seems naive and fragile
    uint32 num_pins = 12;
}

// TODO: serde serialize/deserialize on this
struct SmartCompassPinMessage {
    bytes network_hash = 1 [(nanopb).max_size = 16, (nanopb).fixed_length = true];
    bytes message_hash = 2 [(nanopb).max_size = 16, (nanopb).fixed_length = true];
    uint32 tx_peer_id = 3;

    uint32 last_updated_at = 5;

    int32 latitude = 6;
    int32 longitude = 7;

    uint32 hue = 8; // todo: fixed_length and max_size = 8 bits?
    uint32 saturation = 9; // todo: fixed_length and max_size = 8 bits?
}
*/