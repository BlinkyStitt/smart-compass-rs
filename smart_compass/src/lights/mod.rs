mod networked;
mod patterns;

use crate::ELAPSED_MS;
use crate::periodic::Periodic;
use accelerometer::Orientation;
use smart_leds::{brightness, gamma, SmartLedsWrite, RGB8};

const NUM_LEDS: usize = 256;

/// TODO: do we need this? seems better to just fill the buffre with black
const ALL_BLACK: [RGB8; NUM_LEDS] = [RGB8::new(0, 0, 0); NUM_LEDS];

/// TODO: better trait bounds?
pub struct Lights<SmartLeds: SmartLedsWrite> {
    brightness: u8,
    framerate: Periodic,
    leds: SmartLeds,
    orientation: Orientation,
    last_orientation: Orientation,

    /// use this counter in your patterns
    frames_drawn: u32,

    led_buffer: [RGB8; NUM_LEDS],
}

impl<SmartLeds: SmartLedsWrite> Lights<SmartLeds>
where
    SmartLeds::Color: core::convert::From<smart_leds::RGB<u8>>,
{
    pub fn new(leds: SmartLeds, brightness: u8, frames_per_second: u8) -> Self {
        let light_data: [RGB8; NUM_LEDS] = [RGB8::default(); NUM_LEDS];

        let framerate_ms = 1_000 / (frames_per_second as u32);

        let framerate = Periodic::new(framerate_ms);

        let orientation = Orientation::Unknown;
        let last_orientation = Orientation::Unknown;

        Self {
            brightness,
            framerate,
            frames_drawn: 0,
            leds,
            orientation,
            last_orientation,
            led_buffer: light_data,
        }
    }

    pub fn set_brightness(&mut self, new_brightness: u8) {
        self.brightness = new_brightness;
    }

    pub fn set_orientation(&mut self, new_orientation: Orientation) {
        self.orientation = new_orientation;
    }

    // fill the buffer with the light data
    fn _buffer(&mut self) {
        // TODO: match or something to pick between a bunch of different patterns
        let orientation_changed = self.last_orientation == self.orientation;

        // TODO: have a Pattern state machine that handles orientation and transitionary animations
        match self.orientation {
            Orientation::FaceDown => {
                // render flashlight
                todo!("flashlight pattern");
            }
            Orientation::FaceUp => {
                // render compass
                todo!("compass pattern");
            }
            Orientation::PortraitDown => {
                // render clock
                todo!("clock pattern");
            }
            Orientation::LandscapeUp
            | Orientation::LandscapeDown
            | Orientation::PortraitUp
            | Orientation::Unknown => {
                // render pretty lights
                // TODO: lots of different patterns to choose from
                // TODO: should this use ELAPSED_MS or frames_drawn?
                // TODO: why multiply by 5?
                // TODO: this is flickering all white occasionally
                let j = self.frames_drawn % (NUM_LEDS as u32 * 5);

                for i in 0..NUM_LEDS {
                    self.led_buffer[i] = patterns::wheel(
                        (((i * 256) as u16 / NUM_LEDS as u16 + j as u16) & 255) as u8,
                    );
                }
            }
        };

        if orientation_changed {
            self.last_orientation = self.orientation;
        }
    }

    #[cfg(feature = "lights_interrupt_free")]
    #[inline]
    fn _draw(&mut self) {
        let data = self.led_buffer.clone();

        // correct the colors
        // TODO: do we really need cloned here?
        let data = gamma(data.iter().cloned());

        // dim the lights
        let data = brightness(data, self.brightness);

        // disable interrupts
        cortex_m::interrupt::free(|_| {
            // display
            self.leds.write(data).ok().unwrap();

            // TODO: from a quick test, it looks like drawing 256 WS2812 takes 12-13ms
            // TODO: this should probably be configurable
            unsafe {
                ELAPSED_MS += 12;
            }
        });
    }

    #[cfg(not(feature = "lights_interrupt_free"))]
    #[inline]
    fn _draw(&mut self) {
        let data = self.led_buffer.clone();

        // correct the colors
        // TODO: do we really need cloned here?
        let data = gamma(data.iter().cloned());

        // dim the lights
        let data = brightness(data, self.brightness);

        // display
        self.leds.write(data).ok().unwrap();
    }

    /// TODO: should this fill the buffer with black, too?
    /// TODO: I think FastLED had helpers to do this quickly
    pub fn draw_black(&mut self) {
        self.led_buffer = ALL_BLACK.clone();

        self._draw();
    }

    pub fn draw_test_pattern(&mut self) {
        // 1 red
        self.led_buffer[0].r = 0xFF;

        // 2 green
        self.led_buffer[1].g = 0xFF;
        self.led_buffer[2].g = 0xFF;

        // 3 blue
        self.led_buffer[3].b = 0xFF;
        self.led_buffer[4].b = 0xFF;
        self.led_buffer[5].b = 0xFF;

        // 3 white spread out
        self.led_buffer[6].r = 0xFF;
        self.led_buffer[6].g = 0xFF;
        self.led_buffer[6].b = 0xFF;

        self.led_buffer[8].r = 0xFF;
        self.led_buffer[8].g = 0xFF;
        self.led_buffer[8].b = 0xFF;

        self.led_buffer[255].r = 0xFF;
        self.led_buffer[255].g = 0xFF;
        self.led_buffer[255].b = 0xFF;

        self._draw();
    }

    pub fn draw(&mut self) -> Option<(u32, u32)> {
        if !self.framerate.ready() {
            return None;
        }

        // TODO: warn if framerate is too fast for us to keep up. will need to keep track of the last time we drew

        let start = unsafe { ELAPSED_MS.clone() };

        // fill the light buffer
        // TODO: make it possible to call buffer seperate from draw
        self._buffer();

        // display
        // TODO! some drivers disable interrupts while they draw! this means we won't have an accurate ELAPSED_MS!
        self._draw();

        // increment frames_drawn to advance our patterns
        self.frames_drawn += 1;

        let time = unsafe { ELAPSED_MS.clone() } - start;

        Some((start, time))
    }
}
