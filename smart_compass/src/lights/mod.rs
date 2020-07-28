mod networked;
mod patterns;

use crate::periodic::Periodic;
use accelerometer::Orientation;
// TODO: use smart_leds::gamma
use smart_leds::{brightness, SmartLedsWrite, RGB8};

/// TODO: better trait bounds?
pub struct Lights<SmartLeds: SmartLedsWrite> {
    brightness: u8,
    framerate: Periodic,
    leds: SmartLeds,
    orientation: Orientation,
    last_orientation: Orientation,

    light_data: [RGB8; 256],
}

impl<SmartLeds: SmartLedsWrite> Lights<SmartLeds> {
    pub fn new(leds: SmartLeds, brightness: u8, frames_per_second: u8) -> Self {
        let light_data: [RGB8; 256] = [RGB8::default(); 256];

        let framerate_ms = 1_000 / (frames_per_second as u32);

        let framerate = Periodic::new(framerate_ms);

        let orientation = Orientation::Unknown;
        let last_orientation = Orientation::Unknown;

        Self {
            brightness,
            framerate,
            leds,
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

    /// TODO: i just copied this "where" from the compiler error
    pub fn draw_black(&mut self)
    where
        <SmartLeds as smart_leds::SmartLedsWrite>::Color: core::convert::From<smart_leds::RGB<u8>>,
    {
        static ALL_BLACK: [RGB8; 256] = [RGB8::new(0, 0, 0); 256];

        cortex_m::interrupt::free(|_| {
            self.leds.write(ALL_BLACK.iter().cloned()).ok().unwrap();
        });
    }

    pub fn draw_test_pattern(&mut self)
    where
        <SmartLeds as smart_leds::SmartLedsWrite>::Color: core::convert::From<smart_leds::RGB<u8>>,
    {
        let mut data: [RGB8; 256] = [RGB8::default(); 256];

        data[0].r = 0xFF;
        data[1].g = 0xFF;
        data[2].g = 0xFF;
        data[3].b = 0xFF;
        data[4].b = 0xFF;
        data[5].b = 0xFF;

        data[6].r = 0x80;
        data[6].g = 0x80;
        data[6].b = 0x80;

        data[8].r = 0x80;
        data[8].g = 0x80;
        data[8].b = 0x80;

        data[255].r = 0x80;
        data[255].g = 0x80;
        data[255].b = 0x80;

        // correct colors
        // let data = gamma(data.iter().cloned());

        // dim the lights
        // TODO: do this without cloning?
        let data = brightness(data.iter().cloned(), 16);

        cortex_m::interrupt::free(|_| {
            self.leds.write(data).ok().unwrap();
        });
    }

    /// TODO: return the result instead of unwrapping?
    /// TODO: split this into two functions, one for buffering and one for drawing? (it will need the time that the draw function is expected)
    pub fn draw(&mut self)
    where
        <SmartLeds as smart_leds::SmartLedsWrite>::Color: core::convert::From<smart_leds::RGB<u8>>,
    {
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
        let data = self.light_data;

        // correct colors
        // let data = gamma(data.iter().cloned());

        // dim the lights
        let data = brightness(data.iter().cloned(), my_brightness);

        // display
        // some drivers may need us to disable interrupts, but SPI should work with them
        self.leds.write(data).ok().unwrap();
    }
}
