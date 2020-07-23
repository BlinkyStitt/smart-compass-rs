use radio_sx127x::prelude::*;
use stm32f3_discovery::prelude::*;

use crate::{ELAPSED_MS, MAX_PEERS};
// use blake2::{VarBlake2s};
// use blake2::crypto_mac::{Mac, NewMac};
use cortex_m_semihosting::hprintln;
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
pub struct CompassLocation {
    network_hash: [u8; 16],

    peer_id: usize,
    last_updated_at: u32,
    hue: u8,
    saturation: u8,

    latitude: f32,
    longitude: f32,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    tx_peer_id: usize,
    tx_time: u32,
    tx_ms: u32,

    // TODO: enum for this instead
    location: CompassLocation,
    // TODO: mac of the location message
}

pub struct Network<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay> {
    radio: Sx127x<
        SpiWrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>,
        SpiError,
        PinError,
    >,
    current_mode: Mode,
    // hasher: VarBlake2s,
    my_peer_id: usize,

    // TODO: helpers to change hue/saturation?
    my_hue: u8,
    my_saturation: u8,
    network_hash: [u8; 16],
    locations: [Option<CompassLocation>; MAX_PEERS],
}

impl<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>
    Network<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>
where
    Spi: _embedded_hal_blocking_spi_Transfer<u8, Error = SpiError>
        + _embedded_hal_blocking_spi_Write<u8, Error = SpiError>,
    CsPin: _embedded_hal_digital_OutputPin<Error = PinError>,
    BusyPin: _embedded_hal_digital_InputPin<Error = PinError>,
    ReadyPin: _embedded_hal_digital_InputPin<Error = PinError>,
    ResetPin: _embedded_hal_digital_OutputPin<Error = PinError>,
    Delay: _embedded_hal_blocking_delay_DelayMs<u32>,
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

        let locations = Default::default();

        Self {
            radio,
            current_mode,
            network_hash,
            locations,
            my_peer_id,
            my_hue,
            my_saturation,
        }
    }

    /// TODO: handle multiple types of messages
    pub fn save_message(&mut self, message: Message) {
        let peer_id = message.location.peer_id as usize;

        if peer_id == 0 && self.my_peer_id != 0 {
            // set our millis timer to match the leader's timer
            // TODO: do this better. we have GPS. we should be able to have super accurate time without this
            todo!();
        }

        if let Some(old_location) = &self.locations[peer_id] {
            if old_location.last_updated_at >= message.location.last_updated_at {
                // we already have this message (or a newer one)
                return;
            }
        }

        self.locations[peer_id] = Some(message.location);
    }

    pub fn save_my_location(&mut self, last_updated_at: u32, position: &GpsPosition) {
        match &mut self.locations[self.my_peer_id] {
            Some(compass_location) => {
                compass_location.last_updated_at = last_updated_at;
                compass_location.latitude = position.lat;
                compass_location.longitude = position.lon;
            }
            None => {
                self.locations[self.my_peer_id] = Some(CompassLocation {
                    network_hash: self.network_hash,
                    peer_id: self.my_peer_id,
                    last_updated_at,
                    hue: self.my_hue,
                    saturation: self.my_saturation,
                    latitude: position.lat,
                    longitude: position.lon,
                });
            }
        }
    }

    pub fn transmit(&mut self, peer_id: usize) {
        if self.current_mode == Mode::Transmit {
            if self.radio.check_transmit().ok().unwrap() {
                // another transmission is in process. skip
                return;
            }
        }

        self.current_mode = Mode::Transmit;

        // TODO: reuse the message and serialzer
        let message = Message {
            location: self.locations[peer_id].unwrap().clone(),
            tx_ms: unsafe { ELAPSED_MS },
            tx_peer_id: self.my_peer_id,
            tx_time: 0,
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
                if self.network_hash != message.location.network_hash {
                    // this packet is for a different network
                    return;
                }

                // this packet is for us
                self.save_message(message);
            } else {
                hprintln!("Failed parsing the packet!").unwrap();
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
