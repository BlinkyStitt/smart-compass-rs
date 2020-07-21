//! Adafruit Ultimate GPS Breakout - 66 channel w/10 Hz updates - Version 3
//! https://www.adafruit.com/product/746
//! https://github.com/adafruit/Adafruit_CircuitPython_GPS/blob/master/adafruit_gps.py

// TODO: the adafruit_gps crate requires std::io! looks like we need to roll our own
// TODO: i'd really like to use someone else's code here
// use adafruit_gps::gps::{Gps, GpsSentence};
// use adafruit_gps::send_pmtk::NmeaOutput;
use stm32f3_discovery::prelude::*;

use crate::{hal, GPSSerial, GPS_QUEUE};
use yanp::parse::{GpsDate, GpsPosition, GpsQuality, GpsTime, LongitudeDirection, SentenceData};
use yanp::parse_nmea_sentence;

/// TODO: use generic types instead of hard coding to match our hardware
pub struct UltimateGps {
    // TODO: i think we need to seperate the tx and rx into seperate structs (along with a queue that is currently static)
    tx: hal::serial::Tx<hal::stm32::USART2>,
    rx: hal::serial::Rx<hal::stm32::USART2>,

    /// EN is the Enable pin, it is pulled high with a 10K resistor.
    /// When this pin is pulled to ground, it will turn off the GPS module.
    /// This can be handy for very low power projects where you want to easily turn the module off for long periods.
    /// You will lose your fix if you disable the GPS and it will also take a long time to get fix back if you dont
    /// have the backup battery installed.
    enable_pin: hal::gpio::gpioc::PC6<hal::gpio::Output<hal::gpio::PushPull>>,

    // TODO: what size for buffer_len?
    sentence_buffer_len: usize,
    // TODO: what size? what's the longest sentence?
    sentence_buffer: [u8; 4096],

    data: GpsData,
}

/// There's a lot more information available, but we don't need it right now
#[derive(Default)]
pub struct GpsData {
    pub date: Option<GpsDate>,
    pub magnetic_variation: Option<f32>,
    pub magnetic_direction: Option<LongitudeDirection>,
    pub position: Option<GpsPosition>,
    pub quality: Option<GpsQuality>,
    pub knots: Option<f32>,
    pub heading: Option<f32>,
    pub time: Option<GpsTime>,
    pub sats_in_view: Option<u8>,
}

impl GpsData {
    pub fn update(&mut self, data: SentenceData) -> bool {
        // TODO: support other sentences? GSA for 2d vs 3d fix?
        match data {
            SentenceData::GGA(data) => {
                self.time = data.time;
                self.position = Some(data.position);
                self.quality = data.quality;
                self.sats_in_view = data.sats_in_view;
            }
            SentenceData::RMC(data) => {
                self.time = data.time;
                self.position = Some(data.position);
                self.knots = data.speed;
                self.heading = data.heading;
                self.date = data.date;
                self.magnetic_variation = data.magnetic_variation;
                self.magnetic_direction = data.magnetic_direction;
            }
            _ => return false,
        }

        true
    }
}

impl UltimateGps {
    pub fn new(
        uart: GPSSerial,
        enable_pin: hal::gpio::gpioc::PC6<hal::gpio::Output<hal::gpio::PushPull>>,
    ) -> Self {
        let (tx, rx) = uart.split();

        // TODO: buffer could probably be better
        let sentence_buffer_len = 0;
        let sentence_buffer = [0; 4096];

        let data = GpsData::default();

        Self {
            tx,
            rx,
            enable_pin,
            sentence_buffer_len,
            sentence_buffer,
            data,
        }
    }

    /// Check for updated data from the GPS module and process it accordingly.
    /// Returns True if new data was processed, and False if nothing new was received.
    pub fn update(&mut self) -> bool {
        // NOTE(unsafe) beware of aliasing the `consumer` end point
        let mut consumer = unsafe { GPS_QUEUE.split().1 };

        // pull items off the queue and into our sentence buffer
        // stop looping when the queue is empty or when '\n' is found
        loop {
            // `dequeue` is a lockless operation
            match consumer.dequeue() {
                Some(b) => {
                    self.sentence_buffer[self.sentence_buffer_len] = b;
                    self.sentence_buffer_len += 1;

                    if b == b'\n' {
                        // this is the end of a message!
                        break;
                    }
                }
                None => return false,
            }
        }

        // '\n' was found! we hopefully have a valid sentence

        // TODO: do something with the error?
        let updated = if let Ok(sentence) =
            parse_nmea_sentence(&self.sentence_buffer[0..self.sentence_buffer_len])
        {
            self.data.update(sentence)
        } else {
            false
        };

        // clear the buffer
        self.sentence_buffer_len = 0;

        updated
    }

    /// Send a command string to the GPS.  If add_checksum is True (the
    /// default) a NMEA checksum will automatically be computed and added.
    /// Note you should NOT add the leading $ and trailing * to the command
    /// as they will automatically be added!
    pub fn send_command(&self, command: (), add_checksum: bool) {
        todo!()
    }

    /// True if a current fix for location information is available
    pub fn has_fix(&self) -> bool {
        match self.data.quality {
            Some(GpsQuality::Fix) | Some(GpsQuality::DifferentialFix) => true,
            _ => false,
        }
    }

    pub fn data(&self) -> &GpsData {
        &self.data
    }

    /// Read a byte into the buffer
    /// this gets called inside an interrupt, so make this fast!
    pub fn read(&mut self) {
        // TODO: don't use globals
        // NOTE(unsafe) beware of aliasing the `producer` end point
        let mut producer = unsafe { GPS_QUEUE.split().0 };

        if producer.ready() {
            // the queue has room for another item
            if let Ok(b) = self.rx.read() {
                // NOTE(unsafe) this is fine because...
                // 1. we just checked that the producer is ready
                // 2. this is the only place that calls enqueue
                unsafe {
                    producer.enqueue_unchecked(b);
                }
            }
        }
    }

    pub fn write(&mut self, word: u8) {
        self.tx.write(word).ok().unwrap();
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
