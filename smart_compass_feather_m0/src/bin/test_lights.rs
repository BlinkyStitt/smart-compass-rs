#![no_main]
#![no_std]
#![feature(alloc_error_handler)]
#![feature(asm)]

// panic handler
use panic_halt as _;

pub extern crate feather_m0 as hal;

use hal::prelude::*;
use usb_device::prelude::*;

use alloc_cortex_m::CortexMHeap;
use core::alloc::Layout;
use hal::clock::GenericClockController;
use heapless::consts::*;
use heapless::spsc::{Consumer, Producer, Queue};
use numtoa::NumToA;
use rtic::app;
use smart_compass::{lights, timers};
use usbd_serial::{SerialPort, USB_CLASS_CDC};
use ws2812_timer_delay::Ws2812;

// TODO: do this without allocating (i think its the light test patterns)
#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

type MyLights = lights::Lights<
    Ws2812<hal::timer::TimerCounter3, hal::gpio::Pa20<hal::gpio::Output<hal::gpio::PushPull>>>,
>;

pub type SPIMaster = hal::sercom::SPIMaster4<
    hal::sercom::Sercom4Pad0<hal::gpio::Pa12<hal::gpio::PfD>>,
    hal::sercom::Sercom4Pad2<hal::gpio::Pb10<hal::gpio::PfD>>,
    hal::sercom::Sercom4Pad3<hal::gpio::Pb11<hal::gpio::PfD>>,
>;

// static globals
// the number of ms to offset our network timer. this is time to send+receive+process+draw
// static NETWORK_OFFSET: u16 = 125 + 225;
static DEFAULT_BRIGHTNESS: u8 = 64;

static FRAMES_PER_SECOND: u8 = 75;

/// Quick and dirty way to log messages
pub enum LogMessage {
    RedLedToggle(u32),
    DrawTime(u32, u32),
}

/// TODO: think about this more
fn do_usb_things(
    elapsed_ms: &timers::ElapsedMs,
    usb_device: &mut usb_device::device::UsbDevice<'static, hal::UsbBus>,
    usb_serial: &mut usbd_serial::SerialPort<'static, hal::UsbBus>,
    usb_queue_rx: &mut Consumer<'static, LogMessage, U8, u8>,
) {
    let mut time_buf = [0u8; 32];

    // TODO: read debug commands from serial
    if usb_device.poll(&mut [usb_serial]) {
        let mut msg_buf = [0u8; 64];

        if let Ok(count) = usb_serial.read(&mut msg_buf) {
            let now = elapsed_ms.now();

            if let Ok(_) = usb_serial.write(now.numtoa(10, &mut time_buf)) {
                usb_serial.write(b" - ").ok();

                for (i, c) in msg_buf.iter().enumerate() {
                    if i >= count {
                        break;
                    }
                    // TODO: instead of echoing the command, read the command and take some action based on it
                    usb_serial.write(&[*c]).ok();
                }

                // TODO: newline?
            }
        }
    }

    // TODO: this could be bettr. if writing gets an error, drain the queeue
    while let Some(msg) = usb_queue_rx.dequeue() {
        let now = elapsed_ms.now();

        if let Ok(_) = usb_serial.write(now.numtoa(10, &mut time_buf)) {
            usb_serial.write(b" - ").ok();

            match msg {
                LogMessage::RedLedToggle(start) => {
                    usb_serial.write(b"toggle ").ok();
                    usb_serial.write(start.numtoa(10, &mut time_buf)).ok();
                }
                LogMessage::DrawTime(start, time) => {
                    usb_serial.write(b"draw ").ok();
                    usb_serial.write(start.numtoa(10, &mut time_buf)).ok();
                    usb_serial.write(b" ").ok();
                    usb_serial.write(time.numtoa(10, &mut time_buf)).ok();
                }
            }

            usb_serial.write(b"\n").ok();
        }
    }
}

/// TODO: figure out how to properly use rtic resources
static mut ELAPSED_MS: Option<timers::ElapsedMs> = None;

#[app(device = hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        lights: MyLights,
        elapsed_ms_timer: hal::timer::TimerCounter4,
        every_200_millis: timers::EveryNMillis,
        red_led: hal::gpio::Pa17<hal::gpio::Output<hal::gpio::OpenDrain>>,
        usb_device: usb_device::device::UsbDevice<'static, hal::UsbBus>,
        usb_queue_tx: Producer<'static, LogMessage, U8, u8>,
        usb_queue_rx: Consumer<'static, LogMessage, U8, u8>,
        usb_serial: usbd_serial::SerialPort<'static, hal::UsbBus>,
    }

    /// Increment ELAPSED_MS every millisecond
    /// The `wait()` call is important because it checks and resets the counter ready for the next period.
    #[task(binds = TC4, priority = 3, resources = [elapsed_ms_timer, usb_device, usb_serial, usb_queue_rx])]
    fn tc4(c: tc4::Context) {
        if c.resources.elapsed_ms_timer.wait().is_ok() {
            let elapsed_ms = unsafe { ELAPSED_MS.as_ref().unwrap() };

            elapsed_ms.increment();

            // TODO: get rid of these
            let usb_device = c.resources.usb_device;
            let usb_serial = c.resources.usb_serial;
            let usb_queue_rx = c.resources.usb_queue_rx;

            // TODO: i had this in a task(binds = USB), but that only triggers on receive
            do_usb_things(elapsed_ms, usb_device, usb_serial, usb_queue_rx);
        }
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

        // 3MHz timer for lights
        // TODO: which timer should we use?
        let mut light_timer = hal::timer::TimerCounter::tc3_(
            &clocks.tcc2_tc3(&gclk0).unwrap(),
            device.TC3,
            &mut device.PM,
        );
        light_timer.start(3.mhz());

        // 1ms timer for ELAPSED_MS
        // TODO: which timer should we use?
        let mut elapsed_ms_timer = hal::timer::TimerCounter::tc4_(
            &clocks.tc4_tc5(&gclk0).unwrap(),
            device.TC4,
            &mut device.PM,
        );
        elapsed_ms_timer.start(1.ms());
        elapsed_ms_timer.enable_interrupt();

        // TODO: i can't figure out how to use rtic resources for this
        unsafe {
            ELAPSED_MS = Some(timers::ElapsedMs::default());
        }

        // setup USB serial for debug logging
        // TODO: put these usb things int resources instead of in statics
        let usb_allocator = unsafe {
            static mut USB_ALLOCATOR: Option<usb_device::bus::UsbBusAllocator<hal::UsbBus>> = None;

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

        let usb_serial = SerialPort::new(&usb_allocator);
        let usb_device = UsbDeviceBuilder::new(&usb_allocator, UsbVidPid(0x16c0, 0x27dd))
            .manufacturer("StittHappens")
            .product("Smart Compass")
            .serial_number("TEST")
            .device_class(USB_CLASS_CDC)
            .build();

        let usb_queue = unsafe {
            static mut USB_QUEUE: Option<Queue<LogMessage, U8, u8>> = None;

            USB_QUEUE = Some(Queue::u8());
            USB_QUEUE.as_mut().unwrap()
        };

        let (usb_queue_tx, usb_queue_rx) = usb_queue.split();

        // onboard LED
        let red_led = pins.d13.into_open_drain_output(&mut pins.port);

        // external LEDs
        let light_pin = pins.d6.into_push_pull_output(&mut pins.port);

        let my_lights: MyLights = lights::Lights::new(
            Ws2812::new(light_timer, light_pin),
            DEFAULT_BRIGHTNESS,
            FRAMES_PER_SECOND,
        );

        let every_200_millis = timers::EveryNMillis::new(200);

        init::LateResources {
            every_200_millis,
            lights: my_lights,
            red_led,
            elapsed_ms_timer,
            usb_device,
            usb_queue_tx,
            usb_queue_rx,
            usb_serial,
        }
    }

    #[idle(resources = [
        every_200_millis,
        lights,
        red_led,
        usb_queue_tx,
    ])]
    fn idle(c: idle::Context) -> ! {
        let every_200_millis = c.resources.every_200_millis;
        let my_lights = c.resources.lights;
        let red_led = c.resources.red_led;
        let usb_queue_tx = c.resources.usb_queue_tx;

        let elapsed_ms = unsafe { ELAPSED_MS.as_ref().unwrap() };

        // TODO: reset the usb device?

        // delay.delay_ms(200u16);

        // TODO: why does this need the lock? i thought atomic would mean it wouldn't
        my_lights.draw_black(elapsed_ms);

        // delay.delay_ms(1000u16);
        // my_lights.draw_test_pattern();
        // delay.delay_ms(1000u16);

        loop {
            if let Ok(now) = every_200_millis.ready(elapsed_ms) {
                red_led.toggle();

                // TODO: we used to enqueue_unchecked because if the usb device isn't attached, we can't log to it
                // TODO: but i think our interrupts run fast enough (and they will discard data)
                usb_queue_tx
                    .enqueue(LogMessage::RedLedToggle(now))
                    .ok()
                    .unwrap();

                // TODO: test this
                rtic::pend(hal::pac::Interrupt::USB);
            }

            if let Some((start, time)) = my_lights.draw(elapsed_ms) {
                usb_queue_tx
                    .enqueue(LogMessage::DrawTime(start, time))
                    .ok()
                    .unwrap();

                // TODO: test this. i think doing this means we can drop doing usb check inside the millis interrupt
                rtic::pend(hal::pac::Interrupt::USB);
            }
        }
    }
};

#[alloc_error_handler]
fn oom(_: Layout) -> ! {
    loop {}
}
