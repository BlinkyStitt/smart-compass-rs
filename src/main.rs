#![no_std]
#![no_main]

extern crate feather_m0 as hal;

use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::prelude::*;
use hal::entry;
use hal::pac::{CorePeripherals, Peripherals};
use smart_leds::{brightness, gamma, RGB8, SmartLedsWrite};
use ws2812_spi::Ws2812;

mod battery;
mod compass;
mod config;
mod lights;
mod location;
mod network;
mod sd;

static MAX_PEERS: u8 = 5;
static NETWORK_OFFSET: u16 = 125 + 225;

// TODO: someone has to have a helper for millis()
static mut ELAPSED_MS: u32 = 0;
static mut BATTERY_STATUS: battery::BatteryStatus = battery::BatteryStatus::Full;
// rotating "base color" used by some patterns
static mut G_HUE: u8 = 0;

/// TODO: way too much cargo culting happening here. i'm just copy/pasting. figure out WHY these ethings are what they are
#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let mut core = CorePeripherals::take().unwrap();

    let mut clocks = GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );
    let mut pins = hal::Pins::new(peripherals.PORT);

    let mut delay = Delay::new(core.SYST, &mut clocks);

    // TODO: should we use into_push_pull_output or into_open_drain_output?
    // already wired for us
    let mut rfm95_int = pins.d3.into_pull_down_input(&mut pins.port);
    // already wired for us
    let mut rfm95_rst = pins.d4.into_open_drain_output(&mut pins.port);
    let mut led_data = pins.d5.into_open_drain_output(&mut pins.port);
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

    // shared between radio, sensors, and the SD card
    // TODO: what speed? ws2812-spi says between 2-3.8MHZ. adafruit says way slower though
    let spi = hal::spi_master(
        &mut clocks,
        3.mhz(),
        peripherals.SERCOM4,
        &mut peripherals.PM,
        pins.sck,
        pins.mosi,
        pins.miso,
        &mut pins.port,
    );

    let uart = hal::uart(
        &mut clocks,
        10.mhz(),
        peripherals.SERCOM0,
        &mut peripherals.PM,
        pins.d0,
        pins.d1,
        &mut pins.port,
    );

    // TODO: figure out interrupt timers
    /*
    // gclk0 represents a configured clock using the system 48MHz oscillator
    let gclk0 = clocks.gclk0();
    // configure a clock for the TC4 and TC5 peripherals
    let tc45 = &clocks.tc4_tc5(&gclk0).unwrap();
    // instantiate a timer objec for the TC4 and TC5 peripheral
    let mut timer4 = TimerCounter::tc4_(tc45, peripherals.TC4, &mut peripherals.PM);
    let mut timer5 = TimerCounter::tc5_(tc45, peripherals.TC5, &mut peripherals.PM);
    // TODO: i don't think this is right. someone has to already have a `millis()` helper like arduino has
    timer4.start(48000.hz());
    // TODO: this is what we did on the old code, but i'm not so sure
    timer5.start(10.hz());

    // TODO: the examples do this in a more complicated way:
    // https://github.com/atsamd-rs/atsamd/blob/master/boards/feather_m0/examples/sleeping_timer.rs
    timer4.enable_interrupt();
    timer5.enable_interrupt();
    */

    // create lights
    let mut lights = Ws2812::new(spi);

    let mut light_data: [RGB8; 256] = [RGB8::default(); 256];

    // one red
    light_data[0] = RGB8 {
        r: 0xFF,
        g: 0,
        b: 0,
    };
    // 2 green
    light_data[1] = RGB8 {
        r: 0,
        g: 0xFF,
        b: 0,
    };
    light_data[2] = RGB8 {
        r: 0,
        g: 0xFF,
        b: 0,
    };
    // 3 blue
    light_data[3] = RGB8 {
        r: 0,
        g: 0,
        b: 0xFF,
    };
    light_data[4] = RGB8 {
        r: 0,
        g: 0,
        b: 0xFF,
    };
    light_data[5] = RGB8 {
        r: 0,
        g: 0,
        b: 0xFF,
    };

    let light_data_off: [RGB8; 256] = [RGB8::default(); 256];

    // main loop
    loop {
        delay.delay_ms(200u8);
        red_led.set_high().unwrap();
        lights.write(brightness(gamma(light_data.iter().cloned()), 32)).unwrap();

        delay.delay_ms(200u8);
        red_led.set_low().unwrap();
        lights.write(light_data_off.iter().cloned()).unwrap();
    }
}

// /// keep track of the time we've been running
// /// TODO: arduino does this for us on a 
// #[interrupt]
// fn TC4() {
//     unsafe { ELAPSED_MS += 1 };
// }


// #[interrupt]
// fn TC5() {
//     todo!("gps read");
// }
