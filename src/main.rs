#![no_main]
#![no_std]

// panic handler
use panic_semihosting as _;

// mod battery;
// mod compass;
// mod config;
mod lights;
mod location;
mod network;
mod periodic;
mod storage;

use stm32f3_discovery::prelude::*;

use asm_delay::AsmDelay;
use cortex_m_semihosting::hprintln;
use heapless::mpmc::Q32;
use rtic::app;
use shared_bus_rtic::SharedBus;
use stm32f3_discovery::accelerometer::RawAccelerometer;
use stm32f3_discovery::compass::Compass;
use stm32f3_discovery::hal;
use stm32f3_discovery::leds::Leds as CompassLeds;

/// TODO: what should we name this
pub type SpiMaster = hal::spi::Spi<
    hal::stm32::SPI1,
    (
        hal::gpio::gpioa::PA5<hal::gpio::AF5>,
        hal::gpio::gpioa::PA6<hal::gpio::AF5>,
        hal::gpio::gpioa::PA7<hal::gpio::AF5>,
    ),
>;

/// TODO: what should we name this
type GPSSerial = hal::serial::Serial<
    hal::stm32::USART2,
    (
        hal::gpio::gpiod::PD5<hal::gpio::AF7>,
        hal::gpio::gpiod::PD6<hal::gpio::AF7>,
    ),
>;

/// TODO: what should we name this
type MyLights = lights::Lights<hal::gpio::gpioc::PC5<hal::gpio::Output<hal::gpio::PushPull>>>;

/// TODO: what should we name this

/// TODO: what should we name this
type SdController<SpiWrapper> = embedded_sdmmc::Controller<
    embedded_sdmmc::SdMmcSpi<
        SpiWrapper,
        hal::gpio::gpioc::PC0<hal::gpio::Output<hal::gpio::PushPull>>,
    >,
    storage::DummyTimeSource,
>;

/// TODO: what should we name this
type SpiRadio<SpiWrapper> = network::Radio<
    SpiWrapper,
    hal::spi::Error,
    hal::gpio::gpioc::PC1<hal::gpio::Output<hal::gpio::PushPull>>,
    hal::gpio::gpioc::PC2<hal::gpio::Input<hal::gpio::PullDown>>,
    hal::gpio::gpioc::PC3<hal::gpio::Input<hal::gpio::PullDown>>,
    hal::gpio::gpioc::PC4<hal::gpio::Output<hal::gpio::PushPull>>,
    (),
    asm_delay::AsmDelay,
>;

/// keep everything on the same bus inside one struct
pub struct SharedSPIResources {
    radio: SpiRadio<SharedBus<SpiMaster>>,
    sd_card: SdController<SharedBus<SpiMaster>>,
}

// static globals
// static MAX_PEERS: u8 = 5;
// /// the number of ms to offset our network timer. this is time to send+receive+process+draw
// static NETWORK_OFFSET: u16 = 125 + 225;
static DEFAULT_BRIGHTNESS: u8 = 128;
static FRAMES_PER_SECOND: u8 = 30;

// TODO: use rtic resources instead
static mut ELAPSED_MS: usize = 0;
// TODO: what size queue?

#[app(device = stm32f3_discovery::hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        // TODO: put every_300_seconds into a Battery struct
        every_300_seconds: periodic::Periodic,
        // TODO: put compass in a shared_resources helper if theres more than one i2c
        compass: Compass,
        compass_lights: CompassLeds,
        lights: MyLights,
        gps: location::UltimateGps,
        gps_updater: location::UltimateGpsUpdater,
        shared_spi_resources: SharedSPIResources,
        // timer4: hal::timer::Timer<hal::stm32::TIM4>,
        timer7: hal::timer::Timer<hal::stm32::TIM7>,
    }

    // TODO: low priority?
    // #[task(binds = TIM4, resources = [timer4])]
    // fn tim4(c: tim4::Context) {
    //     if c.resources.timer4.wait().is_ok() {
    //         // c.resources.gps.read();
    //         todo!("read gps");
    //     }
    // }

    #[task(binds = TIM7, resources = [timer7, gps_updater])]
    fn tim7(c: tim7::Context) {
        if c.resources.timer7.wait().is_ok() {
            // TODO: use an atomic for this? or use rtic resources?
            // TODO: is this how arduinio's millis function works?
            unsafe {
                ELAPSED_MS += 1;
            }

            // https://learn.adafruit.com/adafruit-ultimate-gps/parsed-data-output
            // "if you can, get this to run once a millisecond in an interrupt"
            c.resources.gps_updater.read();
        }
    }

    /// setup the hardware
    #[init]
    fn init(c: init::Context) -> init::LateResources {
        // Cortex-M peripherals
        // let core = c.core;

        // Device specific peripherals
        let device = c.device;

        let mut reset_and_clock_control = device.RCC.constrain();

        // setup ITM output
        // TODO: maybe just shove this into a static mut? not sure how to put this into resources
        // let stim = &mut core.ITM.stim[0];

        let mut flash = device.FLASH.constrain();

        // TODO: what speeds should we set?
        let clocks = reset_and_clock_control.cfgr.freeze(&mut flash.acr);

        // pins. this board sure has a lot of them!
        let mut gpioa = device.GPIOA.split(&mut reset_and_clock_control.ahb);
        let mut gpiob = device.GPIOB.split(&mut reset_and_clock_control.ahb);
        let mut gpioc = device.GPIOC.split(&mut reset_and_clock_control.ahb);
        let mut gpiod = device.GPIOD.split(&mut reset_and_clock_control.ahb);
        let mut gpioe = device.GPIOE.split(&mut reset_and_clock_control.ahb);

        // TODO: rtic doesn't expose SYST (link to the github issue about this)
        // let delay = hal::delay::Delay::new(device.SYST, &mut clocks);
        // TODO: get the processor mhz dynamically? `clocks.sysclk()`?
        let delay = AsmDelay::new(asm_delay::bitrate::U32BitrateExt::mhz(72));

        // TODO: what other timers are available? which should we use? i'm just using 7 because thats what the example used
        // TODO: how often should we try to read the gps? the example did every 10hz, but that seems like a lot
        // let mut timer4 = hal::timer::Timer::tim4(
        //     device.TIM4,
        //     10.hz(),
        //     clocks,
        //     &mut reset_and_clock_control.apb1,
        // );
        // timer4.listen(hal::timer::Event::Update);

        // TODO: how many hz to 1 millisecond? i think we have a 72mhz processor, so 72000
        // TODO: calculate this in case we run at a different speed
        let mut timer7 = hal::timer::Timer::tim7(
            device.TIM7,
            72_000.hz(),
            clocks,
            &mut reset_and_clock_control.apb1,
        );
        timer7.listen(hal::timer::Event::Update);

        // TODO: shared-bus for the i2c?
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

        // pick pins that `impl MisoPin<SPI1>`, `impl MosiPin<SPI1>`, `impl SckPin<SPI1>`
        let miso = gpioa.pa6.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
        let mosi = gpioa.pa7.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
        let sck = gpioa.pa5.into_af5(&mut gpioa.moder, &mut gpioa.afrl);

        let spi_mode = hal::spi::Mode {
            polarity: hal::spi::Polarity::IdleLow,
            phase: hal::spi::Phase::CaptureOnFirstTransition,
        };

        // TODO: what frequency?
        let my_spi: SpiMaster = hal::spi::Spi::spi1(
            device.SPI1,
            (sck, miso, mosi),
            spi_mode,
            3.mhz(),
            clocks,
            &mut reset_and_clock_control.apb2,
        );

        // SPI is shared between radio, sensors, and the SD card
        let shared_spi_manager = shared_bus_rtic::new!(my_spi, SpiMaster);

        // setup sd card
        let sd_spi = shared_spi_manager.acquire();

        // TODO: what pin? i picked this randomly
        let sdcard_cs = gpioc
            .pc0
            .into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper);

        let time_source = storage::DummyTimeSource;

        let my_sd_card: SdController<_> = embedded_sdmmc::Controller::new(
            embedded_sdmmc::SdMmcSpi::new(sd_spi, sdcard_cs),
            time_source,
        );

        // setup the radio
        let radio_spi = shared_spi_manager.acquire();

        // TODO: what pins? i picked these randomly
        let rfm95_cs = gpioc
            .pc1
            .into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper);
        let rfm95_busy = gpioc
            .pc2
            .into_pull_down_input(&mut gpioc.moder, &mut gpioc.pupdr);
        let rfm95_ready = gpioc
            .pc3
            .into_pull_down_input(&mut gpioc.moder, &mut gpioc.pupdr);
        let rfm95_reset = gpioc
            .pc4
            .into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper);

        let my_radio: SpiRadio<_> = network::Radio::new(
            radio_spi,
            rfm95_cs,
            rfm95_busy,
            rfm95_ready,
            rfm95_reset,
            delay,
        );

        // TODO: setup orientation sensor

        // setup serial for communicating with the gps module
        // USART1 is connected to ST-LINK, so we use USART2
        // pick pins that `impl RxPin<USART2>` and `impl TxPin<USART2>`
        let gps_tx = gpiod.pd5.into_af7(&mut gpiod.moder, &mut gpiod.afrl);
        let gps_rx = gpiod.pd6.into_af7(&mut gpiod.moder, &mut gpiod.afrl);

        let gps_uart: GPSSerial = hal::serial::Serial::usart2(
            device.USART2,
            (gps_tx, gps_rx),
            9600.bps(),
            clocks,
            &mut reset_and_clock_control.apb1,
        );

        // TODO: what pin shuold we use? this one was random
        let gps_enable_pin = gpioc
            .pc6
            .into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper);

        let (my_gps, my_gps_updater) = location::UltimateGps::new(gps_uart, gps_enable_pin);

        // create lights
        // TODO: what pin shuold we use? this one was random
        let led_data_pin = gpioc
            .pc5
            .into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper);

        let my_lights: MyLights =
            lights::Lights::new(led_data_pin, DEFAULT_BRIGHTNESS, FRAMES_PER_SECOND);

        // TODO: use rtic's periodic tasks instead of our own
        // TODO: should this use the rtc?
        let every_300_seconds = periodic::Periodic::new(300 * 1000);

        // enable interrupts
        // TODO: is there a helper for this?
        unsafe {
            hal::stm32::NVIC::unmask(hal::stm32::Interrupt::TIM4);
            hal::stm32::NVIC::unmask(hal::stm32::Interrupt::TIM7);
        }

        let shared_spi_resources = SharedSPIResources {
            radio: my_radio,
            sd_card: my_sd_card,
        };

        init::LateResources {
            compass: my_compass,
            compass_lights: my_compass_lights,
            every_300_seconds,
            gps: my_gps,
            gps_updater: my_gps_updater,
            lights: my_lights,
            shared_spi_resources,
            timer7,
        }
    }

    // `shared` cannot be accessed from this context
    // TODO: more of this should probably be done with interrupts
    #[idle(resources = [
        compass,
        compass_lights,
        every_300_seconds,
        gps,
        lights,
        shared_spi_resources,
    ])]
    fn idle(c: idle::Context) -> ! {
        let my_compass = c.resources.compass;
        let my_compass_lights = c.resources.compass_lights;
        let my_gps = c.resources.gps;

        loop {
            let accel = my_compass.accel_raw().unwrap();
            let mag = my_compass.mag_raw().unwrap();

            // TODO: should we use hprintln or iprintln?
            // iprintln!(stim, "Accel:{:?}; Mag:{:?}", accel, mag);
            hprintln!("Accel:{:?}; Mag:{:?}", accel, mag).unwrap();

            // TODO: do we need to lock the gps? i think without a lock the interrupt could have issues
            if my_gps.update() {
                hprintln!("GPS updated").unwrap();
            }

            if my_gps.has_fix() {
                hprintln!("GPS has fix").unwrap();
            } else {
                hprintln!("GPS does not have a fix").unwrap();
                // wait on the radio receiving
                // TODO: although maybe that should be in an interrupt?
                // TODO: spend 50% the time with the radio asleep
            }
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
