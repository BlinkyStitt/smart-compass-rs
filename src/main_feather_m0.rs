#![no_main]
#![no_std]

// panic handler
use panic_halt as _;

mod battery;
mod compass;
mod config;
mod lights;
mod location;
mod network;
mod periodic;
mod storage;

pub extern crate feather_m0 as hal;

use hal::prelude::*;

use asm_delay::AsmDelay;
use hal::clock::GenericClockController;
use rtic::app;
use shared_bus_rtic::SharedBus;

pub type SPIMaster = hal::sercom::SPIMaster4<
    hal::sercom::Sercom4Pad0<hal::gpio::Pa12<hal::gpio::PfD>>,
    hal::sercom::Sercom4Pad2<hal::gpio::Pb10<hal::gpio::PfD>>,
    hal::sercom::Sercom4Pad3<hal::gpio::Pb11<hal::gpio::PfD>>,
>;

pub type GPSSerial = hal::sercom::UART0<
    hal::sercom::Sercom0Pad3<hal::gpio::Pa11<hal::gpio::PfC>>,
    hal::sercom::Sercom0Pad2<hal::gpio::Pa10<hal::gpio::PfC>>,
    (),
    (),
>;

// TODO: less strict types here
type SpiRadio<SpiWrapper> = network::Radio<
    SpiWrapper,
    hal::sercom::Error,
    hal::gpio::Pa6<hal::gpio::Output<hal::gpio::PushPull>>,
    hal::gpio::Pb8<hal::gpio::Input<hal::gpio::PullDown>>,
    hal::gpio::Pb9<hal::gpio::Input<hal::gpio::PullDown>>,
    hal::gpio::Pa8<hal::gpio::Output<hal::gpio::PushPull>>,
    (),
    asm_delay::AsmDelay,
>;
pub struct SharedSPIResources {
    radio: SpiRadio<SharedBus<SPIMaster>>,
    sd_controller: embedded_sdmmc::Controller<
        embedded_sdmmc::SdMmcSpi<
            SharedBus<SPIMaster>,
            hal::gpio::Pa18<hal::gpio::Output<hal::gpio::PushPull>>,
        >,
        storage::DummyTimeSource,
    >,
}

// static globals
// static MAX_PEERS: u8 = 5;
// the number of ms to offset our network timer. this is time to send+receive+process+draw
// static NETWORK_OFFSET: u16 = 125 + 225;
static DEFAULT_BRIGHTNESS: u8 = 128;
static FRAMES_PER_SECOND: u8 = 30;

// TODO: use rtic resources instead
static mut ELAPSED_MS: usize = 0;

#[app(device = hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        every_300_seconds: periodic::Periodic,
        lights: lights::Lights<hal::gpio::Pa15<hal::gpio::Output<hal::gpio::PushPull>>>,
        gps: location::Gps,
        shared_spi_resources: SharedSPIResources,
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

    // TODO: low priority?
    #[task(binds = TC4, resources = [timer4, gps])]
    fn tc4(c: tc4::Context) {
        if c.resources.timer4.wait().is_ok() {
            c.resources.gps.read();
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

        // TODO: rtic doesn't expose SYST
        // https://github.com/ryankurte/rust-radio-hal/issues/9#issuecomment-660731913
        // let delay = hal::delay::Delay::new(device.SYST, &mut clocks);
        let delay = AsmDelay::new(asm_delay::bitrate::U32BitrateExt::mhz(48));

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

        // TODO: timer5?

        // setup all the pins
        // TODO: should we use into_push_pull_output or into_open_drain_output?
        // already wired for us
        // TODO: use the interrupt
        let rfm95_interrupt = pins.d3.into_pull_down_input(&mut pins.port);
        // already wired for us
        let rfm95_reset = pins.d4.into_push_pull_output(&mut pins.port);
        let led_data = pins.d5.into_push_pull_output(&mut pins.port);
        // TODO: this pin doesn't actually connect to the radio. is this input type right?
        // already wired for us
        let rfm95_cs = pins.d8.into_push_pull_output(&mut pins.port);
        // already wired for us
        // let vbat_pin = pins.d9.into_floating_input(&mut pins.port); // also analog
        let sdcard_cs = pins.d10.into_push_pull_output(&mut pins.port);
        // let lsm9ds1_csag = pins.d11.into_push_pull_output(&mut pins.port);
        // let lsm9ds1_csm = pins.d12.into_push_pull_output(&mut pins.port);
        // already wired for us
        let red_led = pins.d13.into_open_drain_output(&mut pins.port);
        // wire this to io0
        let rfm95_busy = pins.a1.into_pull_down_input(&mut pins.port);
        // wire this to io1
        let rfm95_ready = pins.a2.into_pull_down_input(&mut pins.port);
        let floating_pin = pins.a3.into_floating_input(&mut pins.port);

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
        // SPI is shared between radio, sensors, and the SD card
        let shared_spi_manager = shared_bus_rtic::new!(my_spi, SPIMaster);

        // setup sd card
        let sd_spi = shared_spi_manager.acquire();

        // TODO: why isn't this working? why does the spi not implement fullduplex?
        let sd_spi = embedded_sdmmc::SdMmcSpi::new(sd_spi, sdcard_cs);

        // TODO: a real time source from tthe rtc?
        let time_source = storage::DummyTimeSource;

        let sd_controller = embedded_sdmmc::Controller::new(
            sd_spi,
            time_source
        );

        // setup the radio
        let radio_spi = shared_spi_manager.acquire();

        let my_radio = network::Radio::new(
            radio_spi,
            rfm95_cs,
            rfm95_busy,
            rfm95_ready,
            rfm95_reset,
            delay,
        );

        // TODO: setup compass/orientation sensor

        // setup serial for communicating with the gps module. 
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

        let my_gps = location::Gps::new(my_uart);

        // create lights
        let my_lights = lights::Lights::new(led_data, DEFAULT_BRIGHTNESS, FRAMES_PER_SECOND);

        // TODO: use rtic's periodic tasks instead of our own
        // TODO: should this use the rtc?
        let every_300_seconds = periodic::Periodic::new(300 * 1000);

        // timer for ELAPSED_MILLIS
        timer3.start(1.ms());
        timer3.enable_interrupt();

        // timer for reading the serial connected to GPS
        timer4.start(10.hz());
        timer4.enable_interrupt();

        let shared_spi_resources = SharedSPIResources {
            radio: my_radio,
            sd_controller,
        };

        init::LateResources {
            every_300_seconds,
            lights: my_lights,
            shared_spi_resources,
            gps: my_gps,
            red_led,
            timer3,
            timer4,
        }
    }

    // `shared` cannot be accessed from this context
    // TODO: more of this should probably be done with interrupts
    #[idle(resources = [
        every_300_seconds,
        gps,
        lights,
        shared_spi_resources,
        red_led,
    ])]
    fn idle(c: idle::Context) -> ! {
        let every_300_seconds = c.resources.every_300_seconds;
        let my_gps = c.resources.gps;
        let my_lights = c.resources.lights;
        let shared_spi_resources = c.resources.shared_spi_resources;
        let red_led = c.resources.red_led;

        red_led.set_high().unwrap();

        loop {
            if every_300_seconds.ready() {
                // TODO: set the brightness based on the battery level
                // TODO: is rounding here okay?
                let new_brightness = match battery::BatteryStatus::check() {
                    battery::BatteryStatus::Dead => DEFAULT_BRIGHTNESS / 2,
                    battery::BatteryStatus::Low => DEFAULT_BRIGHTNESS / 4 * 3,
                    battery::BatteryStatus::Okay => DEFAULT_BRIGHTNESS / 10 * 9,
                    battery::BatteryStatus::Full => DEFAULT_BRIGHTNESS,
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
