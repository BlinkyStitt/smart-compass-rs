/*
shifting

let shift_n = some number calculated from the elapsed time or the number of frames drawn

light_data.iter().cycle().skip(shift_n).take(256).cloned()

TODO: use https://docs.rs/microfft/0.3.0/microfft/ for sound reactive patterns?

// TODO: port these https://github.com/jasoncoon/esp8266-fastled-webserver/blob/fibonacci256/Map.h

*/
use super::super::focalintent::*;
use super::{Pattern, COORDS_X};
use crate::timers::ElapsedMs;
use smart_leds::hsv::{hsv2rgb, Hsv};
use smart_leds::RGB8;

// Draws rainbows with an ever-changing, widely-varying set of parameters.
// https://gist.github.com/kriegsman/964de772d64c502760e5
pub struct Pride {
    elapsed_ms: ElapsedMs,
    pseudotime: u16,
    last_ms: u16,
    hue: u16,
}

impl Pride {
    pub fn new(elapsed_ms: ElapsedMs) -> Self {
        Self {
            elapsed_ms,
            pseudotime: 0,
            last_ms: 0,
            hue: 0,
        }
    }
}

impl Pattern for Pride {
    /// TODO: this is not correct. write tests for beatsin88
    /// TODO: https://github.com/jasoncoon/esp8266-fastled-webserver/blob/fibonacci256/esp8266-fastled-webserver.ino#L1183
    fn draw(&mut self, leds: &mut [RGB8]) {
        let now = self.elapsed_ms.now();

        // TODO: figure out what all these numbers do and make it look good on two concentric rings
        // uint8_t sat8 = beatsin88(87, 220, 250);
        let sat8 = beatsin88(now, 87, 220, 250, 0) as u8;
        // uint8_t brightdepth = beatsin88(341, 96, 224);
        let brightdepth = beatsin88(now, 341, 96, 224, 0) as u8;
        // uint16_t brightnessthetainc16 = beatsin88(203, (25 * 256), (40 * 256));
        let brightnessthetainc16 = beatsin88(now, 203, 25 * 256, 40 * 256, 0);
        // uint8_t msmultiplier = beatsin88(147, 23, 60);
        let msmultiplier = beatsin88(now, 147, 23, 60, 0);

        // uint16_t hue16 = sHue16;
        let mut hue16 = self.hue;
        // uint16_t hueinc16 = beatsin88(113, 1, 3000);
        let hueinc16 = beatsin88(now, 113, 1, 3000, 0);

        // uint16_t ms = network_ms; // this should keep everyone's lights looking the same
        // uint16_t deltams = ms - sLastMillis;
        let deltams = now as u16 - self.last_ms;

        // sLastMillis = ms;
        self.last_ms = now as u16;

        // sPseudotime += deltams * msmultiplier;
        self.pseudotime += deltams * msmultiplier;

        // sHue16 += deltams * beatsin88(400, 5, 9);
        self.hue += deltams * beatsin88(now, 400, 5, 9, 0);

        // uint16_t brightnesstheta16 = sPseudotime;
        let mut brightnesstheta16 = self.pseudotime;

        // for (uint16_t i = 0; i < num_LEDs; i++) {
        for i in COORDS_X.iter().cloned() {
            // hue16 += hueinc16;
            hue16 += hueinc16;
            // uint8_t hue8 = hue16 / 256;
            let hue8: u8 = (hue16 / 256) as u8;

            // brightnesstheta16 += brightnessthetainc16;
            brightnesstheta16 += brightnessthetainc16;
            // uint16_t b16 = sin16(brightnesstheta16) + 32768;
            let b16: u16 = (sin16(brightnesstheta16) + 32767 + 1) as u16;

            // uint16_t bri16 = (uint32_t)((uint32_t)b16 * (uint32_t)b16) / 65536;
            // let bri16: u16 = (((b16 as u32) * (b16 as u32)) / 65536) as u16;

            // uint8_t bri8 = (uint32_t)(((uint32_t)bri16) * brightdepth) / 65536;
            let mut bri8: u8 = (((b16 as u32) * (brightdepth as u32)) / 65536) as u8;
            // bri8 += (255 - brightdepth);
            bri8 += 255 - brightdepth;

            // CRGB newcolor = CHSV(hue8, sat8, bri8);
            let new_color = Hsv {
                hue: hue8,
                sat: sat8,
                val: bri8,
            };

            let new_color = hsv2rgb(new_color);

            // uint16_t pixelnumber = i;
            // pixelnumber = (num_LEDs - 1) - pixelnumber;
            let pixel_number = i as usize;

            // nblend(leds[pixelnumber], newcolor, 64);
            nblend(&mut leds[pixel_number], &new_color, 64);
        }
    }
}
