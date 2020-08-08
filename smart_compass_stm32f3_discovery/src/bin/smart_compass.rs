#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

// panic handler
use panic_semihosting as _;

pub use stm32f3_discovery::prelude::*;

use alloc_cortex_m::CortexMHeap;
use asm_delay::AsmDelay;
use core::alloc::Layout;
use cortex_m_semihosting::hprintln;
use rtic::app;
use shared_bus_rtic::SharedBus;
use smart_compass::{battery, lights, location, network, storage, timers, MAX_PEERS};
use stm32f3_discovery::accelerometer::{Orientation, RawAccelerometer};
use stm32f3_discovery::compass::Compass;
use stm32f3_discovery::cortex_m::asm::delay;
use stm32f3_discovery::cortex_m_rt;
use stm32f3_discovery::hal;
use stm32f3_discovery::leds::Leds as CompassLeds;
use ws2812_spi::Ws2812;

// TODO: i'm not sure what I did to require an allocator
#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

static mut ELAPSED_MS: Option<timers::ElapsedMs> = None;

type MyBattery = battery::Battery<hal::gpio::gpioc::PC8<hal::gpio::Input<hal::gpio::PullDown>>>;

/// TODO: what should we name this
// TODO: less specific type than AF5
pub type MySpi1 = hal::spi::Spi<
    hal::stm32::SPI1,
    (
        hal::gpio::gpioa::PA5<hal::gpio::AF5>,
        hal::gpio::gpioa::PA6<hal::gpio::AF5>,
        hal::gpio::gpioa::PA7<hal::gpio::AF5>,
    ),
>;

pub type MySpi2 = hal::spi::Spi<
    hal::stm32::SPI2,
    (
        hal::gpio::gpiob::PB13<hal::gpio::AF5>,
        hal::gpio::gpiob::PB14<hal::gpio::AF5>,
        hal::gpio::gpiob::PB15<hal::gpio::AF5>,
    ),
>;

/// TODO: what should we name this
type MyLights = lights::Lights<Ws2812<MySpi2>>;

// TODO: move this into the bin somehow. not everythiing will want USART2
/// TODO: what should we name this
type GPSSerial = hal::serial::Serial<
    hal::stm32::USART2,
    (
        hal::gpio::gpiod::PD5<hal::gpio::AF7>,
        hal::gpio::gpiod::PD6<hal::gpio::AF7>,
    ),
>;

type MyGps = location::UltimateGps<
    hal::serial::Tx<hal::stm32::USART2>,
    hal::gpio::PXx<hal::gpio::Output<hal::gpio::OpenDrain>>,
>;

type MyGpsQueue =
    location::UltimateGpsQueue<stm32f3_discovery::hal::serial::Rx<hal::stm32::USART2>>;

/// TODO: what should we name this
type SdController<Spi> = storage::embedded_sdmmc::Controller<
    storage::embedded_sdmmc::SdMmcSpi<
        Spi,
        hal::gpio::gpioc::PC0<hal::gpio::Output<hal::gpio::PushPull>>,
    >,
    storage::DummyTimeSource,
>;

/// TODO: what should we name this
type MyNetwork<Spi> = network::Network<
    Spi,
    hal::spi::Error,
    hal::gpio::PXx<hal::gpio::Output<hal::gpio::PushPull>>,
    hal::gpio::PXx<hal::gpio::Input<hal::gpio::PullDown>>,
    hal::gpio::PXx<hal::gpio::Input<hal::gpio::PullDown>>,
    hal::gpio::PXx<hal::gpio::Output<hal::gpio::OpenDrain>>,
    (),
    asm_delay::AsmDelay,
>;

/// keep everything on the same bus inside one struct
/// notice that the lights are NOT included here. they use SPI, but a different bus!
pub struct SharedSPIResources {
    // TODO: gyroscope on SPI
    network: MyNetwork<SharedBus<MySpi1>>,
    sd_card: SdController<SharedBus<MySpi1>>,
}

// static globals
// TODO: what type?
const NUM_TIME_SEGMENTS: usize = MAX_PEERS * MAX_PEERS;
const TIME_SEGMENT_S: usize = 2;
// /// the number of ms to offset our network timer. this is time to send+receive+process+draw
// static NETWORK_OFFSET: u16 = 125 + 225;
const DEFAULT_BRIGHTNESS: u8 = 128;
const FRAMES_PER_SECOND: u8 = 30;

#[app(device = stm32f3_discovery::hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        battery: MyBattery,
        // TODO: put compass in a shared_resources helper if theres more than one i2c
        compass: Compass,
        compass_lights: CompassLeds,
        elapsed_ms: timers::ElapsedMs,
        elapsed_ms_timer: hal::timer::Timer<hal::stm32::TIM7>,
        lights: MyLights,
        gps: MyGps,
        gps_queue: MyGpsQueue,
        shared_spi_resources: SharedSPIResources,
    }

    #[task(binds = TIM7, resources = [elapsed_ms, elapsed_ms_timer, gps_queue])]
    fn tim7(c: tim7::Context) {
        if c.resources.elapsed_ms_timer.wait().is_ok() {
            // TODO: use an atomic for this? or use rtic resources?
            // TODO: is this how arduinio's millis function works?
            c.resources.elapsed_ms.increment();

            // https://learn.adafruit.com/adafruit-ultimate-gps/parsed-data-output
            // "if you can, get this to run once a millisecond in an interrupt"
            c.resources.gps_queue.read();
        }
    }

    /// setup the hardware
    #[init]
    fn init(c: init::Context) -> init::LateResources {
        // Initialize the allocator BEFORE you use it
        let start = cortex_m_rt::heap_start() as usize;
        let size = 1024; // in bytes
        unsafe { ALLOCATOR.init(start, size) }

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

        // TODO: how many hz to 1 millisecond? i think we have a 72mhz processor, so 72_000
        // TODO: calculate this in case we run at a different speed
        let mut elapsed_ms_timer = hal::timer::Timer::tim7(
            device.TIM7,
            72_000.hz(),
            clocks,
            &mut reset_and_clock_control.apb1,
        );
        // TODO: is this `listen` needed? does rtic handle this for us?
        elapsed_ms_timer.listen(hal::timer::Event::Update);

        // TODO: i wanted a more complex type on this, but i started having troubles with lifetimes
        let elapsed_ms = timers::ElapsedMs::default();

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
        let my_spi: MySpi1 = hal::spi::Spi::spi1(
            device.SPI1,
            (sck, miso, mosi),
            spi_mode,
            24.mhz(),
            clocks,
            &mut reset_and_clock_control.apb2,
        );

        // SPI is shared between radio, sensors, and the SD card
        let shared_spi_manager = shared_bus_rtic::new!(my_spi, MySpi1);

        // setup sd card
        let sd_spi = shared_spi_manager.acquire();

        // TODO: what pin? i picked this randomly
        let sdcard_cs = gpioc
            .pc0
            .into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper);

        let time_source = storage::DummyTimeSource;

        let my_sd_card: SdController<_> = storage::embedded_sdmmc::Controller::new(
            storage::embedded_sdmmc::SdMmcSpi::new(sd_spi, sdcard_cs),
            time_source,
        );

        // setup the radio
        let radio_spi = shared_spi_manager.acquire();

        // TODO: what pins? i picked these randomly
        // TODO: configure the pins in Radio's new function?
        let rfm95_cs = gpioc
            .pc1
            .into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper)
            .downgrade()
            .downgrade();
        // the radio's "DIO0" pin, also known as the IRQ pin
        let rfm95_busy = gpioc
            .pc2
            .into_pull_down_input(&mut gpioc.moder, &mut gpioc.pupdr)
            .downgrade()
            .downgrade();
        // READY is DIO1
        let rfm95_ready = gpioc
            .pc3
            .into_pull_down_input(&mut gpioc.moder, &mut gpioc.pupdr)
            .downgrade()
            .downgrade();
        let rfm95_reset = gpioc
            .pc4
            .into_open_drain_output(&mut gpioc.moder, &mut gpioc.otyper)
            .downgrade()
            .downgrade();

        // TODO: get this from the SD card
        // TODO: 32
        let network_hash = [0u8; 16];
        let my_peer_id = 0;
        let my_hue = 0;
        let my_saturation = 0;

        let my_network: MyNetwork<_> = network::Network::new(
            elapsed_ms,
            radio_spi,
            rfm95_cs,
            rfm95_busy,
            rfm95_ready,
            rfm95_reset,
            delay,
            network_hash,
            my_peer_id,
            my_hue,
            my_saturation,
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

        // TODO: what pin should we use? this one was random
        let gps_enable_pin = gpioc
            .pc6
            .into_open_drain_output(&mut gpioc.moder, &mut gpioc.otyper)
            .downgrade()
            .downgrade();

        let (gps_tx, gps_rx) = gps_uart.split();

        let (my_gps, my_gps_queue) = location::UltimateGps::new(gps_tx, gps_rx, gps_enable_pin);

        // create lights
        // TODO: is spi a good interface for this? whats the best way to run ws2812s?
        // TODO: what pin shuold we use? this one was random
        // pick pins that `impl MisoPin<SPI2>`, `impl MosiPin<SPI2>`, `impl SckPin<SPI2>`
        let miso2 = gpiob.pb14.into_af5(&mut gpiob.moder, &mut gpiob.afrh);
        let mosi2 = gpiob.pb15.into_af5(&mut gpiob.moder, &mut gpiob.afrh);
        let sck2 = gpiob.pb13.into_af5(&mut gpiob.moder, &mut gpiob.afrh);

        let lights_spi: MySpi2 = hal::spi::Spi::spi2(
            device.SPI2,
            (sck2, miso2, mosi2),
            ws2812_spi::MODE,
            3.mhz(),
            clocks,
            &mut reset_and_clock_control.apb1,
        );

        let my_lights: MyLights = lights::Lights::new(
            Ws2812::new(lights_spi),
            DEFAULT_BRIGHTNESS,
            &elapsed_ms,
            None,
            FRAMES_PER_SECOND,
        );

        // TODO: how often should we do this?
        // check the batterry every minute
        let battery = battery::Battery::new(
            gpioc
                .pc8
                .into_pull_down_input(&mut gpioc.moder, &mut gpioc.pupdr),
            &elapsed_ms,
            60_000,
        );

        let shared_spi_resources = SharedSPIResources {
            network: my_network,
            sd_card: my_sd_card,
        };

        init::LateResources {
            battery,
            compass: my_compass,
            compass_lights: my_compass_lights,
            gps: my_gps,
            gps_queue: my_gps_queue,
            lights: my_lights,
            shared_spi_resources,
            elapsed_ms,
            elapsed_ms_timer,
        }
    }

    // `shared` cannot be accessed from this context
    // TODO: more of this should probably be done with interrupts
    #[idle(resources = [
        battery,
        compass,
        compass_lights,
        gps,
        lights,
        shared_spi_resources,
    ])]
    fn idle(c: idle::Context) -> ! {
        let my_battery = c.resources.battery;
        let my_compass = c.resources.compass;
        let my_compass_lights = c.resources.compass_lights;
        let my_gps = c.resources.gps;
        let my_lights = c.resources.lights;
        let shared_spi_resources = c.resources.shared_spi_resources;

        let elapsed_ms = ELAPSED_MS.as_ref().unwrap();

        // rgb test
        my_lights.draw_test_pattern(elapsed_ms);
        elapsed_ms.block(500);

        // a moment of silence
        my_lights.draw_black(elapsed_ms);
        elapsed_ms.block(1500);

        // TODO: read this from the SD
        let my_peer_id = 0;

        my_lights.set_my_peer_id(my_peer_id);

        // configure gps
        // get the version (PMTK_Q_RELEASE)
        my_gps.send_command(b"PMTK605");

        // turn on GPRMC and GGA (PMTK_SET_NMEA_OUTPUT_RMCGGA)
        my_gps.send_command(b"PMTK314,0,1,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0");

        // set the update frequency
        // PMTK_SET_NMEA_UPDATE_1HZ - 1000
        // PMTK_SET_NMEA_UPDATE_5HZ - 200
        // PMTK_SET_NMEA_UPDATE_10HZ - 100
        my_gps.send_command(b"PMTK220,1000");

        hprintln!(
            "Radio silicon version: 0x{:X}",
            shared_spi_resources.network.silicon_version()
        )
        .unwrap();

        // delay for 1 second (TODO: use a helper for calculating 1 second in cycles)
        delay(72_000_000);

        loop {
            match my_battery.check(elapsed_ms) {
                (false, _) => { /* no change */ }
                (true, battery::BatteryStatus::Low) => {
                    hprintln!("Battery low").unwrap();
                    my_lights.set_brightness(DEFAULT_BRIGHTNESS / 2);
                }
                (true, battery::BatteryStatus::Ok) => {
                    hprintln!("Battery ok").unwrap();
                    my_lights.set_brightness(DEFAULT_BRIGHTNESS);
                }
            }

            let accel = my_compass.accel_raw().unwrap();
            let mag = my_compass.mag_raw().unwrap();

            // TODO: should we use hprintln or iprintln?
            // iprintln!(stim, "Accel:{:?}; Mag:{:?}", accel, mag);
            hprintln!("Accel:{:?}; Mag:{:?}", accel, mag).unwrap();

            // TODO: get the actual orientation from a sensor
            // TODO: should this be a global? should it happen on interrupt?
            let orientation = Orientation::Unknown;

            my_lights.set_orientation(orientation);

            {
                let gps_data = my_gps.data();

                my_lights.draw(
                    elapsed_ms,
                    gps_data.time.as_ref(),
                    gps_data.magnetic_variation.as_ref(),
                    shared_spi_resources.network.locations(),
                );
            }

            let mut gps_data;

            if my_gps.receive() {
                hprintln!("GPS received a sentence").unwrap();

                gps_data = my_gps.data();

                if let Some(last_updated_at) = gps_data.epoch_seconds {
                    if let Some(position) = &gps_data.position {
                        shared_spi_resources
                            .network
                            .save_my_location(last_updated_at, position);
                    }
                }
            } else {
                gps_data = my_gps.data();
            }

            my_lights.draw(
                elapsed_ms,
                gps_data.time.as_ref(),
                gps_data.magnetic_variation.as_ref(),
                shared_spi_resources.network.locations(),
            );

            if my_gps.has_fix() {
                hprintln!("GPS has fix").unwrap();

                if let Some(epoch_seconds) = gps_data.epoch_seconds {
                    // TODO: the seconds being a float is really annoying. i don't want to bring floats into this

                    let time_segment_id =
                        (epoch_seconds as usize / TIME_SEGMENT_S) % NUM_TIME_SEGMENTS;

                    let broadcasting_peer_id = time_segment_id / MAX_PEERS;
                    let broadcasted_peer_id = time_segment_id % MAX_PEERS;

                    // radio transmit or receive depending on the time_segment_id
                    // TODO: spend 50% the time with the radio asleep?
                    if broadcasting_peer_id == my_peer_id {
                        // my turn to broadcast
                        shared_spi_resources
                            .network
                            .transmit(time_segment_id, broadcasted_peer_id);
                    } else {
                        // listen for someone else
                        shared_spi_resources.network.try_receive();
                    }
                } else {
                    hprintln!("GPS does not have the time").unwrap();
                    // TODO: should we bother with the radio? maybe put it to sleep?
                    shared_spi_resources.network.sleep();
                }
            } else {
                hprintln!("GPS does not have a fix").unwrap();
                // wait on the radio receiving
                // TODO: although maybe that should be in an interrupt?
            }

            // draw again because the using radio can take a while
            my_lights.draw(
                elapsed_ms,
                gps_data.time.as_ref(),
                gps_data.magnetic_variation.as_ref(),
                shared_spi_resources.network.locations(),
            );

            // TODO: fastLED.delay equivalent to improve brightness at low levels? make sure it doesn't block the radios!
        }
    }
};

#[alloc_error_handler]
fn oom(_: Layout) -> ! {
    loop {}
}
