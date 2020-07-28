#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

// panic handler
use panic_halt as _;

pub extern crate feather_m0 as hal;

use hal::prelude::*;
use usb_device::prelude::*;

use alloc_cortex_m::CortexMHeap;
use asm_delay::AsmDelay;
use core::alloc::Layout;
use hal::clock::GenericClockController;
use numtoa::NumToA;
use rtic::app;
use smart_compass::{lights, periodic, ELAPSED_MS};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

// TODO: i'm not sure what I did to require an allocator
#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

pub type SPIMaster = hal::sercom::SPIMaster4<
    hal::sercom::Sercom4Pad0<hal::gpio::Pa12<hal::gpio::PfD>>,
    hal::sercom::Sercom4Pad2<hal::gpio::Pb10<hal::gpio::PfD>>,
    hal::sercom::Sercom4Pad3<hal::gpio::Pb11<hal::gpio::PfD>>,
>;

// static globals
// the number of ms to offset our network timer. this is time to send+receive+process+draw
// static NETWORK_OFFSET: u16 = 125 + 225;
static DEFAULT_BRIGHTNESS: u8 = 128;
static FRAMES_PER_SECOND: u8 = 30;

// TODO: use rtic resources once i figure out how to handle the static lifetime
static mut USB_ALLOCATOR: Option<usb_device::bus::UsbBusAllocator<hal::UsbBus>> = None;
static mut USB_DEVICE: Option<usb_device::device::UsbDevice<hal::UsbBus>> = None;
static mut USB_SERIAL: Option<usbd_serial::SerialPort<hal::UsbBus>> = None;

#[app(device = hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        lights: lights::Lights<SPIMaster>,
        every_200_millis: periodic::Periodic,
        red_led: hal::gpio::Pa17<hal::gpio::Output<hal::gpio::OpenDrain>>,
        timer4: hal::timer::TimerCounter4,
    }

    /// Increment ELAPSED_MS every millisecond
    /// The `wait()` call is important because it checks and resets the counter ready for the next period.
    #[task(binds = TC4, resources = [timer4], priority = 3)]
    fn tc4(c: tc4::Context) {
        if c.resources.timer4.wait().is_ok() {
            unsafe {
                // TODO: use an rtic resource (atomicUsize?)
                ELAPSED_MS += 1;
            }
        }
    }

    #[task(binds = USB, priority = 1)]
    fn usb(_c: usb::Context) {
        unsafe {
            USB_DEVICE.as_mut().map(|device| {
                USB_SERIAL.as_mut().map(|serial| {
                    device.poll(&mut [serial]);
                    let mut msg_buf = [0u8; 64];

                    if let Ok(count) = serial.read(&mut msg_buf) {
                        // TODO: instead of just echoing, prefix with ELAPSED_MS
                        let mut time_buf = [0u8; 32];
                        serial.write(ELAPSED_MS.numtoa(10, &mut time_buf));

                        serial.write(b" - ").ok().unwrap();

                        for (i, c) in msg_buf.iter().enumerate() {
                            if i >= count {
                                break;
                            }
                            serial.write(&[*c]).ok().unwrap();
                        }
                    };
                    serial
                });
                device
            });
        };
    }

    /// setup the hardware
    #[init]
    fn init(c: init::Context) -> init::LateResources {
        // Initialize the allocator BEFORE you use it
        let start = cortex_m_rt::heap_start() as usize;
        let size = 1024; // in bytes
        unsafe { ALLOCATOR.init(start, size) }

        let mut device = c.device;

        let mut clocks = GenericClockController::with_internal_32kosc(
            device.GCLK,
            &mut device.PM,
            &mut device.SYSCTRL,
            &mut device.NVMCTRL,
        );
        let gclk0 = clocks.gclk0();
        let mut pins = hal::Pins::new(device.PORT);

        // TODO: which timer should we use?
        let mut timer4 = hal::timer::TimerCounter::tc4_(
            &clocks.tc4_tc5(&gclk0).unwrap(),
            device.TC4,
            &mut device.PM,
        );

        // setup USB serial for debug logging
        let usb_allocator = unsafe {
            USB_ALLOCATOR = Some(hal::usb_allocator(
                device.USB,
                &mut clocks,
                &mut device.PM,
                pins.usb_dm,
                pins.usb_dp,
                &mut pins.port,
            ));
            USB_ALLOCATOR.as_ref().unwrap()
        };

        unsafe {
            USB_SERIAL = Some(SerialPort::new(&usb_allocator));
            USB_DEVICE = Some(
                UsbDeviceBuilder::new(&usb_allocator, UsbVidPid(0x16c0, 0x27dd))
                    .manufacturer("Fake company")
                    .product("Serial port")
                    .serial_number("TEST")
                    .device_class(USB_CLASS_CDC)
                    .build(),
            );
        }

        // the ws2812-spi library says between 2-3.8 or something like that
        let my_spi = hal::spi_master(
            &mut clocks,
            3.mhz(),
            device.SERCOM4,
            &mut device.PM,
            pins.sck,
            pins.mosi,
            pins.miso,
            &mut pins.port,
        );

        // onboard LED
        let red_led = pins.d13.into_open_drain_output(&mut pins.port);

        // external LEDs
        let my_lights = lights::Lights::new(my_spi, DEFAULT_BRIGHTNESS, FRAMES_PER_SECOND);

        let every_200_millis = periodic::Periodic::new(200);

        // timer for ELAPSED_MILLIS
        timer4.start(1000.hz());
        timer4.enable_interrupt();

        init::LateResources {
            every_200_millis,
            lights: my_lights,
            red_led,
            timer4,
        }
    }

    #[idle(resources = [
        every_200_millis,
        lights,
        red_led,
    ])]
    fn idle(c: idle::Context) -> ! {
        let every_200_millis = c.resources.every_200_millis;
        let my_lights = c.resources.lights;
        let red_led = c.resources.red_led;

        let mut delay = AsmDelay::new(asm_delay::bitrate::U32BitrateExt::mhz(48));

        my_lights.draw_black();

        delay.delay_ms(1000u16);

        my_lights.draw_test_pattern();

        loop {
            if every_200_millis.ready() {
                red_led.toggle();
                my_lights.draw_test_pattern();
            }

            delay.delay_ms(100u8);
        }
    }
};

#[alloc_error_handler]
fn oom(_: Layout) -> ! {
    loop {}
}
