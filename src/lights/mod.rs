mod networked;
mod patterns;

use feather_m0::prelude::_atsamd_hal_embedded_hal_digital_v2_OutputPin;

use crate::periodic::Periodic;
use accelerometer::Orientation;
use smart_leds::{brightness, gamma, RGB8, SmartLedsWrite};
use ws2812_nop_samd21::Ws2812;

/// TODO: better trait bounds?
pub struct Lights<Pin: _atsamd_hal_embedded_hal_digital_v2_OutputPin> {
    brightness: u8,
    framerate: Periodic,
    lights: Ws2812<Pin>,
    orientation: Orientation,

    pretty_data: [RGB8; 256],
    flashlight_data: [RGB8; 256],
    location_data: [RGB8; 256],
}

impl<Pin: _atsamd_hal_embedded_hal_digital_v2_OutputPin> Lights<Pin> {
    pub fn new(pin: Pin, brightness: u8, frames_per_second: u8) -> Self {
        let lights = Ws2812::new(pin);

        let pretty_data: [RGB8; 256] = [RGB8::default(); 256];
        let flashlight_data: [RGB8; 256] = [RGB8::default(); 256];
        let location_data: [RGB8; 256] = [RGB8::default(); 256];

        // TODO: do this better
        let framerate_ms =  1_000 / (frames_per_second as usize) as usize;

        let framerate = Periodic::new(framerate_ms);

        let orientation = Orientation::Unknown;

        Self {
            brightness,
            framerate,
            lights,
            orientation,
            pretty_data,
            flashlight_data,
            location_data,
        }
    }

    pub fn set_brightness(&mut self, new_brightness: u8) {
        self.brightness = new_brightness;
    }

    pub fn set_orientation(&mut self, new_orientation: Orientation) {
        self.orientation = new_orientation;
    }

    /// TODO: return the result instead of unwrapping?
    pub fn draw(&mut self) {
        let my_brightness = self.brightness;
        
        // get the data
        let data = match self.orientation {
                Orientation::FaceDown => {
                    self.flashlight_data
                },
                Orientation::FaceUp => {
                    self.location_data
                },
                Orientation::LandscapeDown => todo!(),
                Orientation::LandscapeUp => todo!(),
                Orientation::PortraitDown => todo!(),
                Orientation::PortraitUp => todo!(),
                Orientation::Unknown => {
                    self.pretty_data
                },
        };

        // TODO: if framerate period is ready, draw the next frame for this orientation
        if self.framerate.ready() {
            match self.orientation {
                Orientation::FaceDown => todo!(),
                Orientation::FaceUp => todo!(),
                Orientation::LandscapeDown => todo!(),
                Orientation::LandscapeUp => todo!(),
                Orientation::PortraitDown => todo!(),
                Orientation::PortraitUp => todo!(),
                Orientation::Unknown => todo!(),
            };
        }

        // correct colors
        let data = gamma(data.iter().cloned());

        // dim the lights
        let data = brightness(data, my_brightness);

        // display
        self.lights.write(data).unwrap();
    }
}

/*
test pattern:

    let mut light_data: [RGB8; 256] = [RGB8::default(); 256];

    // one red
    light_data[0] = RGB8 {
        r: 0xFF,
        g: 0,
        b: 0,
    };
    // 2 green
    light_data[1] = RGB8 {
        r: 0,
        g: 0xFF,
        b: 0,
    };
    light_data[2] = RGB8 {
        r: 0,
        g: 0xFF,
        b: 0,
    };
    // 3 blue
    light_data[3] = RGB8 {
        r: 0,
        g: 0,
        b: 0xFF,
    };
    light_data[4] = RGB8 {
        r: 0,
        g: 0,
        b: 0xFF,
    };
    light_data[5] = RGB8 {
        r: 0,
        g: 0,
        b: 0xFF,
    };
*/
