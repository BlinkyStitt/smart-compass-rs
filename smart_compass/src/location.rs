//! Adafruit Ultimate GPS Breakout - 66 channel w/10 Hz updates - Version 3
//! https://www.adafruit.com/product/746
//! https://github.com/adafruit/Adafruit_CircuitPython_GPS/blob/master/adafruit_gps.py

// TODO: the adafruit_gps crate requires std::io! looks like we need to roll our own
// TODO: i'd really like to use someone else's code here
// use adafruit_gps::gps::{Gps, GpsSentence};
// use adafruit_gps::send_pmtk::NmeaOutput;
use embedded_hal::digital::v2::OutputPin;
use heapless::consts::U256;
use heapless::spsc::{Consumer, Producer, Queue};
use numtoa::NumToA;
use yanp::parse::{GpsPosition, GpsQuality, LongitudeDirection, SentenceData};
use yanp::parse_nmea_sentence;

/// TODO: use generic types instead of hard coding to match our hardware
pub struct UltimateGps<SerialTx: embedded_hal::serial::Write<u8>, EnablePin: OutputPin> {
    // TODO: is having this be static correct?
    queue_rx: Consumer<'static, u8, U256>,
    serial_tx: SerialTx,

    /// EN is the Enable pin, it is pulled high with a 10K resistor.
    /// When this pin is pulled to ground, it will turn off the GPS module.
    /// This can be handy for very low power projects where you want to easily turn the module off for long periods.
    /// You will lose your fix if you disable the GPS and it will also take a long time to get fix back if you dont
    /// have the backup battery installed.
    enable_pin: EnablePin,

    // TODO: what size for buffer_len?
    sentence_buffer_len: usize,
    // TODO: what size? what's the longest sentence?
    sentence_buffer: [u8; 82],

    epoch: time::PrimitiveDateTime,
    data: GpsData,
}

pub struct UltimateGpsQueue<SerialRx: embedded_hal::serial::Read<u8>> {
    serial_rx: SerialRx,
    queue_tx: Producer<'static, u8, U256>,
}

impl<SerialRx: embedded_hal::serial::Read<u8>> UltimateGpsQueue<SerialRx> {
    /// Read a byte into the queue
    /// this gets called inside an interrupt, so make this fast!
    pub fn read(&mut self) {
        if self.queue_tx.ready() {
            // the queue has room for another item
            if let Ok(b) = self.serial_rx.read() {
                // NOTE(unsafe) this is fine because...
                // 1. we just checked that the producer is ready
                // 2. this is the only place that calls enqueue
                unsafe {
                    self.queue_tx.enqueue_unchecked(b);
                }
            }
        }
    }
}

/// There's a lot more information available, but we don't need it right now
#[derive(Default)]
pub struct GpsData {
    pub date: Option<time::Date>,
    pub magnetic_variation: Option<f32>,
    pub magnetic_direction: Option<LongitudeDirection>,
    pub position: Option<GpsPosition>,
    pub quality: Option<GpsQuality>,
    pub knots: Option<f32>,
    pub heading: Option<f32>,
    pub time: Option<time::Time>,
    pub sats_in_view: Option<u8>,
    pub epoch_seconds: Option<u32>,
}

impl GpsData {
    pub fn update(&mut self, epoch: time::PrimitiveDateTime, data: SentenceData) -> bool {
        // TODO: support other sentences? GSA for 2d vs 3d fix?
        match data {
            SentenceData::GGA(data) => {
                if let Some(gps_time) = data.time {
                    self.time = time::Time::try_from_hms(
                        gps_time.hour,
                        gps_time.minute,
                        gps_time.second as u8,
                    )
                    .ok();
                }
                self.position = Some(data.position);
                self.quality = data.quality;
                self.sats_in_view = data.sats_in_view;
            }
            SentenceData::RMC(data) => {
                if let Some(gps_time) = data.time {
                    self.time = time::Time::try_from_hms(
                        gps_time.hour,
                        gps_time.minute,
                        gps_time.second as u8,
                    )
                    .ok();
                }
                self.position = Some(data.position);
                self.knots = data.speed;
                self.heading = data.heading;
                if let Some(gps_date) = data.date {
                    self.date = time::Date::try_from_ymd(
                        gps_date.year as i32,
                        gps_date.month,
                        gps_date.day,
                    )
                    .ok();
                }
                self.magnetic_variation = data.magnetic_variation;
                self.magnetic_direction = data.magnetic_direction;
            }
            _ => return false,
        }

        // TODO: i'm sure this could be more efficient
        // TODO: use pps_pin on an interrupt to increment this
        if let (Some(gps_date), Some(gps_time)) = (self.date, self.time) {
            let now = time::PrimitiveDateTime::new(gps_date, gps_time);

            let epoch_seconds = (now - epoch).whole_seconds() as u32;

            self.epoch_seconds = Some(epoch_seconds);
        }

        true
    }
}

impl<SerialTx: embedded_hal::serial::Write<u8>, EnablePin: OutputPin>
    UltimateGps<SerialTx, EnablePin>
{
    pub fn new<SerialRx: embedded_hal::serial::Read<u8>>(
        serial_tx: SerialTx,
        serial_rx: SerialRx,
        enable_pin: EnablePin,
        // TODO: `pps_pin` on an interrupt
        // pps_pin:
    ) -> (Self, UltimateGpsQueue<SerialRx>) {
        // `heapless::i` is an "unfortunate implementation detail required to construct heapless types in const context"
        // TODO: do the static outside this?
        static mut Q: Queue<u8, U256> = Queue(heapless::i::Queue::new());

        let (queue_tx, queue_rx) = unsafe { Q.split() };

        // TODO: buffer could probably be better
        let sentence_buffer_len = 0;
        // TODO: how should we handle a buffer overflow?
        let sentence_buffer = [0; 82];

        let data = GpsData::default();

        let epoch_date = time::date!(2020 - 06 - 30);
        let epoch_time = time::time!(0:00);

        let epoch = time::PrimitiveDateTime::new(epoch_date, epoch_time);

        let gps = Self {
            queue_rx,
            serial_tx,
            enable_pin,
            sentence_buffer_len,
            sentence_buffer,
            data,
            epoch,
        };

        let updater = UltimateGpsQueue {
            serial_rx,
            queue_tx,
        };

        (gps, updater)
    }

    /// Check for updated data from the GPS module and process it accordingly.
    /// Returns True if new data was processed, and False if nothing new was received.
    pub fn receive(&mut self) -> bool {
        // pull items off the queue and into our sentence buffer
        // stop looping when the queue is empty or when '\n' is found
        loop {
            // `dequeue` is a lockless operation
            match self.queue_rx.dequeue() {
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
            self.data.update(self.epoch, sentence)
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
    pub fn send_command(&mut self, command: &[u8]) {
        self.write(b'$');

        let mut checksum = 0u8;

        for b in command.iter() {
            self.write(*b);
            checksum ^= b;
        }

        let mut checksum_buf = [0u8; 2];

        let checksum = checksum.numtoa(16, &mut checksum_buf);

        for b in checksum.iter() {
            self.write(*b);
        }

        self.write(b'\r');
        self.write(b'\n');
    }

    /// Turn off the module.
    /// You will lose your fix if you disable the GPS
    /// It will also take a long time to get fix back if you dont have the backup battery installed.
    pub fn disable(&mut self) {
        self.enable_pin.set_low().ok().unwrap();
    }

    /// Turn the module on
    pub fn enable(&mut self) {
        self.enable_pin.set_high().ok().unwrap();
    }

    /// True if a current fix for location information is available
    pub fn has_fix(&self) -> bool {
        match self.data.quality {
            Some(GpsQuality::Fix) | Some(GpsQuality::DifferentialFix) => true,
            _ => false,
        }
    }

    #[inline(always)]
    pub fn data(&self) -> &GpsData {
        &self.data
    }

    #[inline(always)]
    pub fn write(&mut self, word: u8) {
        self.serial_tx.write(word).ok().unwrap();
    }

    pub fn try_time(&self) -> &Option<time::Time> {
        &self.data.time
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
