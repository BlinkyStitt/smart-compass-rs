/*
shifting

let shift_n = some number calculated from the elapsed time or the number of frames drawn

light_data.iter().cycle().skip(shift_n).take(256).cloned()

TODO: use https://docs.rs/microfft/0.3.0/microfft/ for sound reactive patterns?

// TODO: port these https://github.com/jasoncoon/esp8266-fastled-webserver/blob/fibonacci256/Map.h

*/
use super::super::focalintent::*;
use super::{Pattern, COORDS_X};
use smart_leds::hsv::{hsv2rgb, Hsv};
use smart_leds::RGB8;

// Draws rainbows with an ever-changing, widely-varying set of parameters.
// https://gist.github.com/kriegsman/964de772d64c502760e5
// TODO: instead of u16s, use proper types
pub struct Pride {
    pseudotime: u16,
    last_ms: u16,
    hue: u16,
    saturation_bpm: u16,
    saturation_min: u16,
    saturation_max: u16,
    bright_depth_bpm: u16,
    bright_depth_min: u16,
    bright_depth_max: u16,
    bright_theta_inc_bpm: u16,
    bright_theta_inc_min: u16,
    bright_theta_inc_max: u16,
    ms_multiplier_bpm: u16,
    ms_multiplier_min: u16,
    ms_multiplier_max: u16,
    hue_inc_bpm: u16,
    hue_inc_min: u16,
    hue_inc_max: u16,
    s_hue_bpm: u16,
    s_hue_min: u16,
    s_hue_max: u16,
}

impl Pride {
    pub fn new() -> Self {
        Self {
            pseudotime: 0,
            last_ms: 0,
            hue: 0,
            saturation_bpm: 87,
            saturation_min: 220,
            saturation_max: 250,
            bright_depth_bpm: 341,
            bright_depth_min: 96,
            bright_depth_max: 224,
            bright_theta_inc_bpm: 203,
            bright_theta_inc_min: 25 * 256,
            bright_theta_inc_max: 40 * 256,
            ms_multiplier_bpm: 147,
            ms_multiplier_min: 23,
            ms_multiplier_max: 60,
            hue_inc_bpm: 113,
            hue_inc_min: 1,
            hue_inc_max: 3000,
            s_hue_bpm: 400,
            s_hue_min: 5,
            s_hue_max: 9,
        }
    }
}

impl Pattern for Pride {
    /// TODO: this is not correct. write tests for beatsin88
    /// TODO: https://github.com/jasoncoon/esp8266-fastled-webserver/blob/fibonacci256/esp8266-fastled-webserver.ino#L1183
    fn draw(&mut self, now: u32, leds: &mut [RGB8]) {
        // TODO: figure out what all these numbers do and make it look good on two concentric rings
        // TODO: the "official" pattern uses beatsin88 here, but that returns a u16, not a u8
        // uint8_t sat8 = beatsin88(87, 220, 250);
        let sat8 = beatsin88(
            self.saturation_bpm,
            self.saturation_min,
            self.saturation_max,
            now,
            0,
        ) as u8;
        // uint8_t brightdepth = beatsin88(341, 96, 224);
        let bright_depth = beatsin88(
            self.bright_depth_bpm,
            self.bright_depth_min,
            self.bright_depth_max,
            now,
            0,
        ) as u8;
        // uint16_t brightnessthetainc16 = beatsin88(203, (25 * 256), (40 * 256));
        let bright_theta_inc = beatsin88(
            self.bright_theta_inc_bpm,
            self.bright_theta_inc_min,
            self.bright_theta_inc_max,
            now,
            0,
        );
        // uint8_t msmultiplier = beatsin88(147, 23, 60);
        let ms_multiplier = beatsin88(
            self.ms_multiplier_bpm,
            self.ms_multiplier_min,
            self.ms_multiplier_max,
            now,
            0,
        );

        // uint16_t hue16 = sHue16;
        let mut hue16 = self.hue;
        // uint16_t hueinc16 = beatsin88(113, 1, 3000);
        let hueinc16 = beatsin88(self.hue_inc_bpm, self.hue_inc_min, self.hue_inc_max, now, 0);

        // uint16_t ms = network_ms; // this should keep everyone's lights looking the same
        // uint16_t deltams = ms - sLastMillis;
        let deltams = now as u16 - self.last_ms;

        // sLastMillis = ms;
        self.last_ms = now as u16;

        // sPseudotime += deltams * msmultiplier;
        self.pseudotime += deltams * ms_multiplier;

        // sHue16 += deltams * beatsin88(400, 5, 9);
        self.hue += deltams * beatsin88(self.s_hue_bpm, self.s_hue_min, self.s_hue_max, now, 0);

        // uint16_t brightnesstheta16 = sPseudotime;
        let mut bright_theta = self.pseudotime;

        // for (uint16_t i = 0; i < num_LEDs; i++) {
        for i in COORDS_X.iter().cloned() {
            // hue16 += hueinc16;
            hue16 += hueinc16;
            // uint8_t hue8 = hue16 / 256;
            let hue8: u8 = (hue16 / 256) as u8;

            // brightnesstheta16 += brightnessthetainc16;
            bright_theta += bright_theta_inc;
            // uint16_t b16 = sin16(brightnesstheta16) + 32768;
            // TODO: better way to wrap around
            // TODO: why does sin8 return a u8, but sin16 returns a i16? seems like it should be a u16
            let b16: u16 = (sin16(bright_theta).wrapping_add(32767).wrapping_add(1)) as u16;

            // uint16_t bri16 = (uint32_t)((uint32_t)b16 * (uint32_t)b16) / 65536;
            let bri16 = (((b16 as u32) * (b16 as u32)) / 65536) as u16;

            // uint8_t bri8 = (uint32_t)(((uint32_t)bri16) * brightdepth) / 65536;
            let mut bri8: u8 = (((bri16 as u32) * (bright_depth as u32)) / 65536) as u8;

            // bri8 += (255 - brightdepth);
            bri8 += 255 - bright_depth;

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
