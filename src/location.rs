use adafruit_gps::gps::{Gps, open_port, GpsSentence};
use adafruit_gps::send_pmtk::NmeaOutput;

// TODO: the old code read the gps data on a timer. do we want that still?
// https://github.com/atsamd-rs/atsamd/blob/master/boards/feather_m0/examples/timers.rs
