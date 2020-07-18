mod networked;
mod patterns;

use feather_m0::prelude::_atsamd_hal_embedded_hal_digital_v2_OutputPin;

use crate::hal::cortex_m::interrupt;
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

    light_data: [RGB8; 256],
}

impl<Pin: _atsamd_hal_embedded_hal_digital_v2_OutputPin> Lights<Pin> {
    pub fn new(pin: Pin, brightness: u8, frames_per_second: u8) -> Self {
        let lights = Ws2812::new(pin);

        let light_data: [RGB8; 256] = [RGB8::default(); 256];

        let framerate_ms = 1_000 / (frames_per_second as usize);

        let framerate = Periodic::new(framerate_ms);

        let orientation = Orientation::Unknown;

        Self {
            brightness,
            framerate,
            lights,
            orientation,
            light_data,
        }
    }

    pub fn set_brightness(&mut self, new_brightness: u8) {
        self.brightness = new_brightness;
    }

    pub fn set_orientation(&mut self, new_orientation: Orientation) {
        self.orientation = new_orientation;
    }

    pub fn draw_test_pattern(&mut self) {
        todo!("draw 1 red, 2 green, 3 blue and delay for 1 second");
    }

    /// TODO: return the result instead of unwrapping?
    pub fn draw(&mut self) {
        static mut LAST_ORIENTATION: Orientation = Orientation::Unknown;

        let my_brightness = self.brightness;
        
        // TODO: if framerate period is ready, draw the next frame for this orientation
        if self.framerate.ready() {
            let orientation_changed = unsafe {
                LAST_ORIENTATION == self.orientation
            };

            match self.orientation {
                Orientation::FaceDown => {
                    // render flashlight
                    // self.update_flashlight(orientation_changed);
                },
                Orientation::FaceUp => {
                    // render compass
                    // self.update_compass(orientation_changed);
                },
                Orientation::PortraitDown => {
                    // render clock
                    // self.update_clock(orientation_changed);
                },
                Orientation::LandscapeUp | Orientation::LandscapeDown | Orientation::PortraitUp | Orientation::Unknown => {
                    // render pretty lights
                    // self.update_pretty_lights(orientation_changed);
                },
            };

            if orientation_changed {
                unsafe {
                    LAST_ORIENTATION = self.orientation;
                }
            }
        }

        // get the data
        let data = self.light_data.clone();

        // correct colors
        let data = gamma(data.iter().cloned());

        // dim the lights
        let data = brightness(data, my_brightness);

        // display (without interrupts)
        interrupt::free(|_cs| {
            self.lights.write(data).unwrap();
        })
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
