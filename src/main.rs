#![no_main]
#![no_std]

// panic handler
use panic_halt as _;

mod battery;
mod compass;
mod config;
mod lights;
// mod location;
mod network;
mod periodic;
mod sd;

pub extern crate feather_m0 as hal;

use hal::prelude::*;

use hal::clock::GenericClockController;
use rtic::app;

// static globals
static MAX_PEERS: u8 = 5;
// the number of ms to offset our network timer. this is time to send+receive+process+draw
static NETWORK_OFFSET: u16 = 125 + 225;
static DEFAULT_BRIGHTNESS: u8 = 128;
static FRAMES_PER_SECOND: u8 = 30;

// TODO: use rtic resources instead
static mut ELAPSED_MS: usize = 0;

#[app(device = hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        every_300_seconds: periodic::Periodic,
        lights: lights::Lights<hal::gpio::Pa15<hal::gpio::Output<hal::gpio::PushPull>>>,
        red_led: hal::gpio::Pa17<hal::gpio::Output<hal::gpio::OpenDrain>>,
        timer3: hal::timer::TimerCounter3,
        timer4: hal::timer::TimerCounter4,
    }

    /// This function is called each time the tc3 interrupt triggers.
    /// We use it to toggle the LED.  The `wait()` call is important
    /// because it checks and resets the counter ready for the next
    /// period.
    /// TODO: is this how arduino's millis function works?
    #[task(binds = TC3, resources = [timer3])]
    fn tc3(c: tc3::Context) {
        if c.resources.timer3.wait().is_ok() {
            unsafe {
                ELAPSED_MS += 1;
            }
        }
    }

    /// setup the hardware
    #[init]
    fn init(c: init::Context) -> init::LateResources {
        let mut device = c.device;

        let mut clocks = GenericClockController::with_internal_32kosc(
            device.GCLK,
            &mut device.PM,
            &mut device.SYSCTRL,
            &mut device.NVMCTRL,
        );
        let gclk0 = clocks.gclk0();
        let mut pins = hal::Pins::new(device.PORT);

        let mut timer3 = hal::timer::TimerCounter::tc3_(
            &clocks.tcc2_tc3(&gclk0).unwrap(),
            device.TC3,
            &mut device.PM,
        );

        let mut timer4 = hal::timer::TimerCounter::tc4_(
            &clocks.tc4_tc5(&gclk0).unwrap(),
            device.TC4,
            &mut device.PM,
        );

        // timer for ELAPSED_MILLIS
        timer3.start(1.ms());
        timer3.enable_interrupt();

        // timer for reading serial connected to GPS
        timer4.start(10.hz());
        timer4.enable_interrupt();

        // TODO: interrupts for reading from the radio?

        // setup all the pins
        // TODO: should we use into_push_pull_output or into_open_drain_output?
        // already wired for us
        let mut rfm95_int = pins.d3.into_pull_down_input(&mut pins.port);
        // already wired for us
        let mut rfm95_rst = pins.d4.into_push_pull_output(&mut pins.port);
        let mut led_data = pins.d5.into_push_pull_output(&mut pins.port);
        // TODO: this pin doesn't actually connect to the radio. is this input type right?
        let mut rfm95_busy_fake = pins.d6.into_pull_down_input(&mut pins.port);
        // already wired for us
        let mut rfm95_cs = pins.d8.into_push_pull_output(&mut pins.port);
        // already wired for us
        let mut vbat_pin = pins.d9.into_floating_input(&mut pins.port); // also analog
        let mut sdcard_cs = pins.d10.into_push_pull_output(&mut pins.port);
        let mut lsm9ds1_csag = pins.d11.into_push_pull_output(&mut pins.port);
        let mut lsm9ds1_csm = pins.d12.into_push_pull_output(&mut pins.port);
        // already wired for us
        let mut red_led = pins.d13.into_open_drain_output(&mut pins.port);
        let mut floating_pin = pins.a0.into_floating_input(&mut pins.port);

        // SPI is shared between radio, sensors, and the SD card
        // TODO: what speed? m4 maxes at 24mhz. is m0 the same? what 
        let my_spi = hal::spi_master(
            &mut clocks,
            24.mhz(),
            device.SERCOM4,
            &mut device.PM,
            pins.sck,
            pins.mosi,
            pins.miso,
            &mut pins.port,
        );

        // setup serial for communicating with the gps module
        // TOOD: SERCOM0 or SERCOM1?
        // TODO: what speed?
        let my_uart = hal::uart(
            &mut clocks,
            9600.hz(),
            device.SERCOM0,
            &mut device.PM,
            pins.d0,
            pins.d1,
            &mut pins.port,
        );

        // create lights
        // TODO: 
        let my_lights = lights::Lights::new(led_data, DEFAULT_BRIGHTNESS, FRAMES_PER_SECOND);

        // TODO: setup radio
        // TODO: what should delay be?
        // TODO: i think that we need shared-bus to share the spi
        // let mut my_radio = network::Radio::new(my_spi, rfm95_cs, rfm95_busy_fake, rfm95_int, rfm95_rst, delay);

        // TODO: setup compass/orientation sensor
        // TODO: setup sd card

        // TODO: setup setup gps
        // TODO: the adafruit_gps crate requires std::io! looks like we need to roll our own
        // let gps = location::new_gps(uart);

        // TODO: use rtic's periodic tasks instead of our own
        // TODO: should this use the rtc?
        let every_300_seconds = periodic::Periodic::new(300 * 1000);

        // TODO: should we use into_push_pull_output or into_open_drain_output?
        init::LateResources {
            every_300_seconds,
            lights: my_lights,
            // rfm95_int: pins.d3.into_pull_down_input(&mut pins.port),
            // already wired for us
            // rfm95_rst: pins.d4.into_push_pull_output(&mut pins.port),
            // led_data: pins.d5.into_push_pull_output(&mut pins.port),
            // TODO: this pin doesn't actually connect to the radio. is this input type right?
            // rfm95_busy_fake: pins.d6.into_pull_down_input(&mut pins.port),
            // already wired for us
            // rfm95_cs: pins.d8.into_push_pull_output(&mut pins.port),
            // already wired for us
            // vbat_pin: pins.d9.into_floating_input(&mut pins.port), // d9 is also analog
            // sdcard_cs: pins.d10.into_push_pull_output(&mut pins.port),
            // lsm9ds1_csag: pins.d11.into_push_pull_output(&mut pins.port),
            // lsm9ds1_csm: pins.d12.into_push_pull_output(&mut pins.port),          
            // already wired for us
            red_led,
            // floating_pin: pins.a0.into_floating_input(&mut pins.port),
            timer3,
            timer4,
        }
    }

    // `shared` cannot be accessed from this context
    #[idle(resources = [
        every_300_seconds,
        lights,
    ])]
    fn idle(c: idle::Context) -> ! {
        let every_300_seconds = c.resources.every_300_seconds;
        let my_lights = c.resources.lights;

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
    
            my_lights.draw();
    
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
            my_lights.draw();
    
            // TODO: fastLED.delay equivalent to improve brightness? make sure it doesn't block the radios!
        }
    }
};
