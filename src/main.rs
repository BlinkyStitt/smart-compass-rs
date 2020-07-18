#![no_std]
#![no_main]

pub extern crate feather_m0 as hal;

use hal::prelude::*;

use panic_halt as _; // panic handler

use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::exception;
use hal::clock::GenericClockController;
use hal::entry;
use hal::pac::{CorePeripherals, Peripherals};

mod battery;
mod compass;
mod config;
mod lights;
// mod location;
mod network;
mod periodic;
mod sd;

// static globals
static MAX_PEERS: u8 = 5;
// the number of ms to offset our network timer. this is time to send+receive+process+draw
static NETWORK_OFFSET: u16 = 125 + 225;
static DEFAULT_BRIGHTNESS: u8 = 128;
static FRAMES_PER_SECOND: u8 = 30;

// static mut globals. modifying these is UNSAFE!
// TODO: this should probably be a type that hides the unsafe usage from us
pub static mut ELAPSED_MS: usize = 0;

/// TODO: way too much cargo culting happening here. i'm just copy/pasting. figure out WHY these ethings are what they are
#[entry]
fn main() -> ! {
    // setup hardware
    let mut peripherals = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();

    {
        let mut syst = core.SYST;

        // configures the system timer to trigger a SysTick exception every millisecnd
        syst.set_clock_source(SystClkSource::Core);
        // this is configured for the feather_m0 which has a default CPU clock of 48 MHz
        syst.set_reload(48_000);
        syst.clear_current();
        syst.enable_counter();
        syst.enable_interrupt();
    }

    let mut clocks = GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );
    let mut pins = hal::Pins::new(peripherals.PORT);

    // setup all the pins
    // TODO: should we use into_push_pull_output or into_open_drain_output?
    // already wired for us
    let mut rfm95_int = pins.d3.into_pull_down_input(&mut pins.port);
    // already wired for us
    let mut rfm95_rst = pins.d4.into_open_drain_output(&mut pins.port);
    let mut led_data = pins.d5.into_open_drain_output(&mut pins.port);
    // TODO: this pin doesn't actually connect to the radio. is this input type right?
    let mut rfm95_busy_fake = pins.d6.into_pull_down_input(&mut pins.port);
    // already wired for us
    let mut rfm95_cs = pins.d8.into_open_drain_output(&mut pins.port);
    // already wired for us
    let mut vbat_pin = pins.d9.into_floating_input(&mut pins.port); // also analog
    let mut sdcard_cs = pins.d10.into_open_drain_output(&mut pins.port);
    let mut lsm9ds1_csag = pins.d11.into_open_drain_output(&mut pins.port);
    let mut lsm9ds1_csm = pins.d12.into_open_drain_output(&mut pins.port);
    // already wired for us
    let mut red_led = pins.d13.into_open_drain_output(&mut pins.port);
    let mut floating_pin = pins.a0.into_floating_input(&mut pins.port);

    // SPI is shared between radio, sensors, and the SD card
    // TODO: what speed? ws2812-spi says between 2-3.8MHZ. adafruit says way slower though
    let my_spi = hal::spi_master(
        &mut clocks,
        3.mhz(),
        peripherals.SERCOM4,
        &mut peripherals.PM,
        pins.sck,
        pins.mosi,
        pins.miso,
        &mut pins.port,
    );

    // setup serial for communicating with the gps module
    // TOOD: SERCOM0 or SERCOM1?
    let my_uart = hal::uart(
        &mut clocks,
        10.mhz(),
        peripherals.SERCOM0,
        &mut peripherals.PM,
        pins.d0,
        pins.d1,
        &mut pins.port,
    );

    // create lights
    let mut my_lights = lights::Lights::new(my_spi, DEFAULT_BRIGHTNESS, FRAMES_PER_SECOND);

    // TODO: setup radio
    // TODO: what should delay be?
    let mut my_radio = network::Radio::new(my_spi, rfm95_cs, rfm95_busy_fake, rfm95_int, rfm95_rst);

    // TODO: setup compass/orientation sensor
    // TODO: setup sd card

    // TODO: setup setup gps
    // TODO: the adafruit_gps crate requires std::io! looks like we need to roll our own
    // let gps = location::new_gps(uart);

    let mut every_300_seconds = periodic::Periodic::new(300 * 1000);

    // main loop
    loop {
        if every_300_seconds.ready() {
            // TODO: set the brightness based on the battery level
            // TODO: is rounding here okay?
            let new_brightness = match battery::BatteryStatus::check() {
                battery::BatteryStatus::Dead => {
                    DEFAULT_BRIGHTNESS / 2
                },
                battery::BatteryStatus::Low => {
                    DEFAULT_BRIGHTNESS / 4 * 3
                },
                battery::BatteryStatus::Okay => {
                    DEFAULT_BRIGHTNESS / 10 * 9
                },
                battery::BatteryStatus::Full => {
                    DEFAULT_BRIGHTNESS
                },
            };

            my_lights.set_brightness(new_brightness);
        }

        // TODO: get the actual orientation from a sensor
        // TODO: should this be a global?
        let orientation = accelerometer::Orientation::Unknown;

        my_lights.set_orientation(orientation);

        // TODO: get location from the GPS

        my_lights.draw().unwrap();

        // TODO: if we have a GPS fix, 
        if false {
            // TODO: get the time from the GPS

            // TODO: get the time_segment_id

            // TODO: get the broadcasting_peer_id and broadcasted_peer_id

            // TODO: radio transmit or receive depending on the time_segment_id
        } else {
            // TODO: radio receive
        }

        // draw again because the using radio can take a while
        my_lights.draw().unwrap();

        // TODO: fastLED.delay equivalent to improve brightness? make sure it doesn't block the radios!
    }
}

// Exception handler for the SysTick (System Timer) exception
// TODO: how do we do this without unsafe? a mutex? we don't have atomics
#[exception]
fn SysTick() {
    // TODO: likely race condition here!
    // i'd use atomics, but the m0 doesn't look like it supports them
    // TODO: https://docs.rust-embedded.org/book/concurrency/
    unsafe { ELAPSED_MS += 1 };
}

// TODO: interrupt for updating GPS
