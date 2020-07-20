#![no_main]
#![no_std]

// panic handler
use panic_semihosting as _;

// mod battery;
// mod compass;
// mod config;
// mod lights;
// mod location;
// mod network;
mod periodic;
// mod storage;

use stm32f3_discovery::prelude::*;

use stm32f3_discovery::accelerometer::RawAccelerometer;
use stm32f3_discovery::compass::Compass;
use stm32f3_discovery::hal;
use stm32f3_discovery::leds::Leds as CompassLeds;

use asm_delay::AsmDelay;
use rtic::app;
use shared_bus_rtic::SharedBus;

/*

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

type SpiRadio<SpiWrapper> = network::Radio<
    SpiWrapper,
    hal::sercom::Error,
    hal::gpio::Pa6<hal::gpio::Output<hal::gpio::PushPull>>,
    hal::gpio::Pa20<hal::gpio::Input<hal::gpio::PullDown>>,
    hal::gpio::Pa9<hal::gpio::Input<hal::gpio::PullDown>>,
    hal::gpio::Pa8<hal::gpio::Output<hal::gpio::PushPull>>,
    (),
    asm_delay::AsmDelay,
>;
pub struct SharedSPIResources {
    radio: SpiRadio<SharedBus<SPIMaster>>,
}
*/

// static globals
// static MAX_PEERS: u8 = 5;
// the number of ms to offset our network timer. this is time to send+receive+process+draw
// static NETWORK_OFFSET: u16 = 125 + 225;
static DEFAULT_BRIGHTNESS: u8 = 128;
static FRAMES_PER_SECOND: u8 = 30;

// TODO: use rtic resources instead. or at least an atomic
static mut ELAPSED_MS: usize = 0;

#[app(device = stm32f3_discovery::hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        // every_300_seconds: periodic::Periodic,
        // TODO: put this in a shared_resources helper if theres more than one i2c
        compass: Compass,
        compass_lights: CompassLeds,
        // lights: lights::Lights<hal::gpio::Pa15<hal::gpio::Output<hal::gpio::PushPull>>>,
        // gps: location::Gps,
        // shared_spi_resources: SharedSPIResources,
        // red_led: hal::gpio::Pa17<hal::gpio::Output<hal::gpio::OpenDrain>>,
        timer4: hal::timer::Timer<hal::stm32::TIM4>,
        timer7: hal::timer::Timer<hal::stm32::TIM7>,
    }

    // TODO: low priority?
    #[task(binds = TIM4, resources = [timer4])]
    fn tim4(c: tim4::Context) {
        if c.resources.timer4.wait().is_ok() {
            // c.resources.gps.read();
            todo!("read gps");
        }
    }

    /// TODO: is this how arduino's millis function works?
    #[task(binds = TIM7, resources = [timer7])]
    fn tim7(c: tim7::Context) {
        if c.resources.timer7.wait().is_ok() {
            unsafe {
                ELAPSED_MS += 1;
            }
        }
    }

    /// setup the hardware
    #[init]
    fn init(c: init::Context) -> init::LateResources {
        // Cortex-M peripherals
        let core = c.core;

        // Device specific peripherals
        let device = c.device;

        let mut reset_and_clock_control = device.RCC.constrain();

        // setup ITM output
        // TODO: maybe just shove this into a static mut? not sure how to put this into resources
        // let stim = &mut core.ITM.stim[0];

        let mut flash = device.FLASH.constrain();

        let clocks = reset_and_clock_control.cfgr.freeze(&mut flash.acr);

        // pins
        let mut gpiob = device.GPIOB.split(&mut reset_and_clock_control.ahb);
        let mut gpioe = device.GPIOE.split(&mut reset_and_clock_control.ahb);

        // TODO: rtic doesn't expose SYST
        // let delay = hal::delay::Delay::new(device.SYST, &mut clocks);
        // TODO: get the processor mhz dynamically? `clocks.sysclk()`?
        let delay = AsmDelay::new(asm_delay::bitrate::U32BitrateExt::mhz(72));

        // TODO: what other timers are available? which should we use? i'm just using 7 because thats what the example used
        // TODO: how often should we try to read the gps? the example did every 10hz, but that seems like a lot
        let mut timer4 = hal::timer::Timer::tim4(
            device.TIM4,
            10.hz(),
            clocks,
            &mut reset_and_clock_control.apb1,
        );
        timer4.listen(hal::timer::Event::Update);

        // TODO: how many hz to 1 millisecond? i think we have a 72mhz processor, so 72000
        let mut timer7 = hal::timer::Timer::tim7(
            device.TIM7,
            72_000.hz(),
            clocks,
            &mut reset_and_clock_control.apb1,
        );
        timer7.listen(hal::timer::Event::Update);

        // // setup all the pins
        // // TODO: should we use into_push_pull_output or into_open_drain_output?
        // // already wired for us
        // let rfm95_int = pins.d3.into_pull_down_input(&mut pins.port);
        // // already wired for us
        // let rfm95_rst = pins.d4.into_push_pull_output(&mut pins.port);
        // let led_data = pins.d5.into_push_pull_output(&mut pins.port);
        // // TODO: this pin doesn't actually connect to the radio. is this input type right?
        // let rfm95_busy_fake = pins.d6.into_pull_down_input(&mut pins.port);
        // // already wired for us
        // let rfm95_cs = pins.d8.into_push_pull_output(&mut pins.port);
        // // already wired for us
        // // let vbat_pin = pins.d9.into_floating_input(&mut pins.port); // also analog
        // let sdcard_cs = pins.d10.into_push_pull_output(&mut pins.port);
        // // let lsm9ds1_csag = pins.d11.into_push_pull_output(&mut pins.port);
        // // let lsm9ds1_csm = pins.d12.into_push_pull_output(&mut pins.port);
        // // already wired for us
        // let red_led = pins.d13.into_open_drain_output(&mut pins.port);
        // // let floating_pin = pins.a0.into_floating_input(&mut pins.port);

        // new lsm303 driver uses continuous mode, so no need wait for interrupts on DRDY
        let my_compass = Compass::new(
            gpiob.pb6,
            gpiob.pb7,
            &mut gpiob.moder,
            &mut gpiob.afrl,
            device.I2C1,
            clocks,
            &mut reset_and_clock_control.apb1,
        )
        .unwrap();

        // initialize user leds
        let my_compass_lights = CompassLeds::new(
            gpioe.pe8,
            gpioe.pe9,
            gpioe.pe10,
            gpioe.pe11,
            gpioe.pe12,
            gpioe.pe13,
            gpioe.pe14,
            gpioe.pe15,
            &mut gpioe.moder,
            &mut gpioe.otyper,
        );

        // TODO: what speed? m4 maxes at 24mhz. is m0 the same? what
        // let my_spi = hal::spi_master(
        //     &mut clocks,
        //     24.mhz(),
        //     device.SERCOM4,
        //     &mut device.PM,
        //     pins.sck,
        //     pins.mosi,
        //     pins.miso,
        //     &mut pins.port,
        // );
        // // SPI is shared between radio, sensors, and the SD card
        // let shared_spi_manager = shared_bus_rtic::new!(my_spi, SPIMaster);

        // // setup sd card
        // // let sd_spi = shared_spi_manager.acquire();

        // // TODO: why isn't this working? why does the spi not implement fullduplex?
        // // let sd_spi = embedded_sdmmc::SdMmcSpi::new(sd_spi, sdcard_cs);

        // // let time_source = storage::DummyTimeSource;
        // // let mut cont = embedded_sdmmc::Controller::new(
        // //     sd_spi,
        // //     time_source
        // // );

        // // setup the radio
        // let radio_spi = shared_spi_manager.acquire();

        // let my_radio = network::Radio::new(
        //     radio_spi,
        //     rfm95_cs,
        //     rfm95_busy_fake,
        //     rfm95_int,
        //     rfm95_rst,
        //     delay,
        // );

        // // TODO: setup compass/orientation sensor

        // // setup serial for communicating with the gps module
        // // TOOD: SERCOM0 or SERCOM1?
        // // TODO: what speed?
        // let my_uart = hal::uart(
        //     &mut clocks,
        //     9600.hz(),
        //     device.SERCOM0,
        //     &mut device.PM,
        //     pins.d0,
        //     pins.d1,
        //     &mut pins.port,
        // );

        // let my_gps = location::Gps::new(my_uart);

        // // create lights
        // let my_lights = lights::Lights::new(led_data, DEFAULT_BRIGHTNESS, FRAMES_PER_SECOND);

        // // TODO: use rtic's periodic tasks instead of our own
        // // TODO: should this use the rtc?
        // let every_300_seconds = periodic::Periodic::new(300 * 1000);

        // timer for ELAPSED_MILLIS
        unsafe {
            hal::stm32::NVIC::unmask(hal::stm32::Interrupt::TIM4);
            hal::stm32::NVIC::unmask(hal::stm32::Interrupt::TIM7);
        }

        // let shared_spi_resources = SharedSPIResources { radio: my_radio, sd: my_sd };

        init::LateResources {
            compass: my_compass,
            compass_lights: my_compass_lights,
            // stim,
            // every_300_seconds,
            // lights: my_lights,
            // shared_spi_resources,
            // gps: my_gps,
            // red_led,
            timer4,
            timer7,
        }
    }

    // `shared` cannot be accessed from this context
    // TODO: more of this should probably be done with interrupts
    #[idle(resources = [
        compass,
        compass_lights,
        // every_300_seconds,
        // gps,
        // lights,
        // shared_spi_resources,
        // red_led,
    ])]
    fn idle(c: idle::Context) -> ! {
        let my_compass = c.resources.compass;
        // let my_compass_lights = c.resources.compass_lights.into_array();

        loop {
            let accel = my_compass.accel_raw().unwrap();
            let mag = my_compass.mag_raw().unwrap();
            // iprintln!(stim, "Accel:{:?}; Mag:{:?}", accel, mag);

            wait_for_interrupt();
        }

        /*
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
        */
    }
};
