use radio_sx127x::prelude::*;

use crate::MAX_PEERS;
// use blake2::{VarBlake2s};
// use blake2::crypto_mac::{Mac, NewMac};
// use cortex_m_semihosting::hprintln;
use crate::timers::ElapsedMs;
use serde::{Deserialize, Serialize};
use serde_cbor::ser::SliceWrite;
use serde_cbor::Serializer;
use yanp::parse::GpsPosition;

#[derive(PartialEq)]
enum Mode {
    Sleep,
    Transmit,
    Receive,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct PeerLocation {
    pub network_hash: [u8; 16],

    pub peer_id: usize,
    pub last_updated_at: u32,
    pub hue: u8,
    pub sat: u8,

    pub lat: f32,
    pub lon: f32,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    tx_peer_id: usize,
    tx_time: u32,
    tx_ms: u32,

    // TODO: enum for this instead
    location: PeerLocation,
    // TODO: mac of the location message
}

type MyRadio<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay> = Sx127x<
    SpiWrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>,
    SpiError,
    PinError,
>;

/// the usize is the broadcasted_at_id that this was last broadcast at. i don't love this pattern, but it keeps us from broadcasting a message multiple times in a short timespan
pub type PeerLocations = [Option<(PeerLocation, usize)>; MAX_PEERS];

#[derive(Default)]
pub struct NetworkData {
    pub my_peer_id: usize,
    pub my_hue: u8,
    pub my_saturation: u8,
    pub network_hash: [u8; 16],
    pub peer_locations: PeerLocations,
}

pub struct Network<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay> {
    /// TODO: use the radio::Radio trait and do Network<Radio>
    radio: MyRadio<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>,
    current_mode: Mode,
    // hasher: VarBlake2s,
    pub data: NetworkData,
}

impl<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>
    Network<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>
where
    Spi: embedded_hal::blocking::spi::Transfer<u8, Error = SpiError>
        + embedded_hal::blocking::spi::Write<u8, Error = SpiError>,
    CsPin: embedded_hal::digital::v2::OutputPin<Error = PinError>,
    BusyPin: embedded_hal::digital::v2::InputPin<Error = PinError>,
    ReadyPin: embedded_hal::digital::v2::InputPin<Error = PinError>,
    ResetPin: embedded_hal::digital::v2::OutputPin<Error = PinError>,
    Delay: embedded_hal::blocking::delay::DelayMs<u32>,
{
    pub fn new(
        spi: Spi,
        cs: CsPin,
        busy: BusyPin,
        ready: ReadyPin,
        reset: ResetPin,
        delay: Delay,
        network_hash: [u8; 16],
        my_peer_id: usize,
        my_hue: u8,
        my_saturation: u8,
    ) -> Self {
        // TODO: what config?
        let config = Config::default();

        let radio = Sx127x::spi(spi, cs, busy, ready, reset, delay, &config)
            .ok()
            .unwrap();

        let current_mode = Mode::Sleep;

        // let hasher = VarBlake2s::new_keyed(network_key, 16);

        let data = NetworkData {
            my_peer_id,
            my_hue,
            my_saturation,
            network_hash,
            ..Default::default()
        };

        Self {
            radio,
            current_mode,
            data,
        }
    }

    /// TODO: handle multiple types of messages
    pub fn save_message(&mut self, message: Message) {
        let peer_id = message.location.peer_id as usize;

        if peer_id == 0 {
            // unconfigured. ignore this message
            return;
        }

        if peer_id == 1 && self.data.my_peer_id != 1 {
            // set our millis timer to match the leader's timer
            // TODO: do this better. we have GPS. we should be able to have super accurate time without this
            // TODO: maybe instead we should just use the oldest millis
            todo!();
        }

        if let Some((old_location, _)) = &self.data.peer_locations[peer_id] {
            if old_location.last_updated_at >= message.location.last_updated_at {
                // we already have this message (or a newer one)
                return;
            }
        }

        self.data.peer_locations[peer_id] = Some((message.location, 0));
    }

    pub fn save_my_location(&mut self, last_updated_at: u32, position: &GpsPosition) {
        match &mut self.data.peer_locations[self.data.my_peer_id] {
            Some((compass_location, broadcast_at)) => {
                compass_location.last_updated_at = last_updated_at;
                compass_location.lat = position.lat;
                compass_location.lon = position.lon;

                *broadcast_at = 0;
            }
            None => {
                let location = PeerLocation {
                    network_hash: self.data.network_hash,
                    peer_id: self.data.my_peer_id,
                    last_updated_at,
                    hue: self.data.my_hue,
                    sat: self.data.my_saturation,
                    lat: position.lat,
                    lon: position.lon,
                };

                self.data.peer_locations[self.data.my_peer_id] = Some((location, 0));
            }
        }
    }

    pub fn transmit(&mut self, elapsed_ms: &ElapsedMs, time_segment_id: usize, peer_id: usize) {
        if self.current_mode == Mode::Transmit && self.radio.check_transmit().ok().unwrap() {
            // another transmission is in process. skip
            return;
        } else {
            self.current_mode = Mode::Transmit;
        }

        if let Some((location, broadcasted_at_id)) = &mut self.data.peer_locations[peer_id] {
            if *broadcasted_at_id <= time_segment_id {
                // we've already broadcast this transaction
                return;
            }

            *broadcasted_at_id = time_segment_id;

            let now = elapsed_ms.now();

            // TODO: reuse the message and serialzer
            let message = Message {
                location: *location,
                tx_ms: now,
                tx_peer_id: self.data.my_peer_id,
                tx_time: now,
            };

            let mut buf = [0u8; 255];
            let writer = SliceWrite::new(&mut buf[..]);
            let mut ser = Serializer::new(writer);
            message
                .serialize(&mut ser)
                .expect("Failed serializing message for transmission");

            self.radio.start_transmit(&buf).ok().unwrap();

            // TODO: mark this data as transmitted
            // TODO: block until transmission is complete?
        }
    }

    pub fn try_receive(&mut self) {
        // TODO: only do this if we aren't already in receive mode!
        if self.current_mode != Mode::Receive {
            self.radio.start_receive().ok().unwrap();
            self.current_mode = Mode::Receive;
        }

        // TODO: true or false here?
        if self.radio.check_receive(true).ok().unwrap() {
            // TODO: what is the maximum packet length? I think its 255
            let mut buff = [0u8; 1024];
            let mut info = PacketInfo::default();

            let n = self.radio.get_received(&mut info, &mut buff).ok().unwrap();

            let data: Result<Message, _> = serde_cbor::de::from_mut_slice(&mut buff[0..n]);

            if let Ok(message) = data {
                if self.data.network_hash != message.location.network_hash {
                    // this packet is for a different network
                    return;
                }

                // this packet is for us
                self.save_message(message);
            } else {
                // hprintln!("Failed parsing the packet!").unwrap();
            }

            todo!();
            // match std::str::from_utf8(&buff[0..n as usize]) {
            //     Ok(s) => info!("Received: '{}' info: {:?}", s, info),
            //     Err(_) => info!("Received: '{:?}' info: {:?}", &buff[0..n as usize], info),
            // }
        }
    }

    pub fn sleep(&mut self) {
        if self.current_mode != Mode::Sleep {
            self.radio.set_state(State::Sleep).ok().unwrap();
        }
    }

    pub fn silicon_version(&mut self) -> u8 {
        self.radio.silicon_version().ok().unwrap()
    }
}
