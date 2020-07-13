#![no_std]
#![no_main]

extern crate feather_m0 as hal;

use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::prelude::*;
use hal::entry;
use hal::pac::{CorePeripherals, Peripherals};

mod battery;
mod compass;
mod config;
mod lights;
mod location;
mod network;
mod sd;

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();

    let mut clocks = GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );
    let mut pins = hal::Pins::new(peripherals.PORT);

    let mut delay = Delay::new(core.SYST, &mut clocks);

    /*
    // Pins 0 (Pa11) and 1 (Pa10) are used for Serial1 (GPS)
    #define RFM95_INT 3 // already wired for us. PA09
    #define RFM95_RST 4 // already wired for us. PA08
    #define LED_DATA 5  // PA15
    #define RFM95_CS 8      // already wired for us. PA06
    #define VBAT_PIN 9      // already wired for us  // A7. PA07
    #define SDCARD_CS 10    // PA18
    #define LSM9DS1_CSAG 11 // PA16
    #define LSM9DS1_CSM 12  // PA19
    #define RED_LED 13      // already wired for us. PA17. rust d13
    #define FLOATING_PIN 14  // this is diffrent than the prototype
    #define SPI_MISO 22     // shared between Radio+Sensors+SD
    #define SPI_MOSI 23     // shared between Radio+Sensors+SD
    #define SPI_SCK 24      // shared between Radio+Sensors+SD
    */
    let mut serial_rx = pins.rx.into_pull_down_input(&mut pins.port);
    let mut serial_tx = pins.tx.into_push_pull_output(&mut pins.port);
    let mut rfm95_int = pins.d3.into_pull_down_input(&mut pins.port);
    let mut rfm95_rst = pins.d4.into_push_pull_output(&mut pins.port);
    let mut led_data = pins.d5.into_push_pull_output(&mut pins.port);
    let mut rfm95_cs = pins.d8.into_push_pull_output(&mut pins.port);
    let mut floating_pin = pins.a0.into_floating_input(&mut pins.port);
    let mut vbat_pin = pins.d9.into_floating_input(&mut pins.port); // also analog
    let mut sdcard_cs = pins.d10.into_push_pull_output(&mut pins.port);
    let mut lsm9ds1_csag = pins.d11.into_push_pull_output(&mut pins.port);
    let mut lsm9ds1_csm = pins.d12.into_push_pull_output(&mut pins.port);
    let mut red_led = pins.d13.into_open_drain_output(&mut pins.port);

    // TODO: this is wrong: https://github.com/stm32-rs/stm32f0xx-hal/issues/71
    let mut spi_miso = pins.miso.into_pull_down_input(&mut pins.port);
    let mut spi_mosi = pins.mosi.into_push_pull_output(&mut pins.port);
    let mut spi_sck = pins.sck.into_push_pull_output(&mut pins.port);

    // main loop
    loop {
        delay.delay_ms(200u8);
        red_led.set_high().unwrap();
        delay.delay_ms(200u8);
        red_led.set_low().unwrap();
    }
}
