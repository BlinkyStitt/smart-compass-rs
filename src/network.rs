use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use embedded_spi::wrapper::{Wrapper as SpiWrapper};
use radio_sx127x::prelude::*;

use feather_m0::cortex_m::hal

pub struct Radio<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay> {
    radio: Sx127x<SpiWrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>, SpiError, PinError>,
}

impl<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>
    Radio<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>
where
    Spi: Transfer<u8, Error = SpiError> + Write<u8, Error = SpiError>,
    CsPin: OutputPin<Error = PinError>,
    BusyPin: InputPin<Error = PinError>,
    // TODO: should ReadyPin have a where?
    ResetPin: OutputPin<Error = PinError>,
    Delay: DelayMs<u32>,
{
    pub fn new(
        spi: Spi,
        cs: CsPin,
        busy: BusyPin,
        ready: ReadyPin,
        reset: ResetPin,
    ) -> Self {
        // TODO: what config?
        let config = Config::default();

        // TODO: i don't know what to pass here
        let delay = Default::default();

        let radio = Sx127x::spi(spi, cs, busy, ready, reset, delay, &config).unwrap_or_else(|_| panic!());

        Self {
            radio
        }
    }
}
