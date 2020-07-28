mod networked;
mod patterns;

pub use ws2812_spi;

use crate::periodic::Periodic;
use accelerometer::Orientation;
use embedded_hal::spi;
use smart_leds::{brightness, gamma, SmartLedsWrite, RGB8};
use ws2812_spi::Ws2812;

/// TODO: better trait bounds?
pub struct Lights<Spi: spi::FullDuplex<u8>> {
    brightness: u8,
    framerate: Periodic,
    lights: Ws2812<Spi>,
    orientation: Orientation,
    last_orientation: Orientation,

    light_data: [RGB8; 256],
}

impl<Spi: spi::FullDuplex<u8>> Lights<Spi> {
    pub fn new(spi: Spi, brightness: u8, frames_per_second: u8) -> Self {
        let lights = Ws2812::new(spi);

        let light_data: [RGB8; 256] = [RGB8::default(); 256];

        let framerate_ms = 1_000 / (frames_per_second as u32);

        let framerate = Periodic::new(framerate_ms);

        let orientation = Orientation::Unknown;
        let last_orientation = Orientation::Unknown;

        Self {
            brightness,
            framerate,
            lights,
            orientation,
            last_orientation,
            light_data,
        }
    }

    pub fn set_brightness(&mut self, new_brightness: u8) {
        self.brightness = new_brightness;
    }

    pub fn set_orientation(&mut self, new_orientation: Orientation) {
        self.orientation = new_orientation;
    }

    pub fn update_flashlight(&mut self, orientation_changed: bool) {
        todo!();
    }

    pub fn update_compass(&mut self, orientation_changed: bool) {
        todo!();
    }

    pub fn update_clock(&mut self, orientation_changed: bool) {
        todo!();
    }

    pub fn update_pretty_lights(&mut self, orientation_changed: bool) {
        todo!();
    }

    pub fn draw_black(&mut self) {
        let data: [RGB8; 256] = [RGB8::new(0, 0, 0); 256];

        self.lights.write(data.iter().cloned()).ok().unwrap();
    }

    pub fn draw_test_pattern(&mut self) {
        let mut data: [RGB8; 256] = [RGB8::default(); 256];

        data[0] = RGB8 {
            r: 0xFF,
            g: 0,
            b: 0,
        };
        data[1] = RGB8 {
            r: 0,
            g: 0xFF,
            b: 0,
        };
        data[2] = RGB8 {
            r: 0,
            g: 0xFF,
            b: 0,
        };
        data[3] = RGB8 {
            r: 0,
            g: 0,
            b: 0xFF,
        };
        data[4] = RGB8 {
            r: 0,
            g: 0,
            b: 0xFF,
        };
        data[5] = RGB8 {
            r: 0,
            g: 0,
            b: 0xFF,
        };
        data[6] = RGB8 {
            r: 0x80,
            g: 0x80,
            b: 0x80,
        };
        data[8] = RGB8 {
            r: 0x80,
            g: 0x80,
            b: 0x80,
        };
        data[10] = RGB8 {
            r: 0x80,
            g: 0x80,
            b: 0x80,
        };
        data[255] = RGB8 {
            r: 0x80,
            g: 0x80,
            b: 0x80,
        };

        // correct colors
        // let data = gamma(data.iter().cloned());

        // dim the lights
        let data = brightness(data.iter().cloned(), 16);

        // TODO: do this without cloning?
        self.lights.write(data).ok().unwrap();
    }

    /// TODO: return the result instead of unwrapping?
    /// TODO: split this into two functions, one for buffering and one for drawing? (it will need the time that the draw function is expected)
    pub fn draw(&mut self) {
        if !self.framerate.ready() {
            return;
        }

        let my_brightness = self.brightness;

        let orientation_changed = self.last_orientation == self.orientation;

        match self.orientation {
            Orientation::FaceDown => {
                // render flashlight
                self.update_flashlight(orientation_changed);
            }
            Orientation::FaceUp => {
                // render compass
                self.update_compass(orientation_changed);
            }
            Orientation::PortraitDown => {
                // render clock
                self.update_clock(orientation_changed);
            }
            Orientation::LandscapeUp
            | Orientation::LandscapeDown
            | Orientation::PortraitUp
            | Orientation::Unknown => {
                // render pretty lights
                self.update_pretty_lights(orientation_changed);
            }
        };

        if orientation_changed {
            self.last_orientation = self.orientation;
        }

        // get the data
        let data = &self.light_data;

        // correct colors
        // let data = gamma(data.iter().cloned());

        // dim the lights
        let data = brightness(data.iter().cloned(), my_brightness);

        // display
        // some drivers may need us to disable interrupts, but SPI should work with them
        self.lights.write(data).ok().unwrap();
    }
}
