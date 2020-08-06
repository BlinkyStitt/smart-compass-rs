/*
shifting

let shift_n = some number calculated from the elapsed time or the number of frames drawn

light_data.iter().cycle().skip(shift_n).take(256).cloned()

TODO: use https://docs.rs/microfft/0.3.0/microfft/ for sound reactive patterns?

// TODO: port these https://github.com/jasoncoon/esp8266-fastled-webserver/blob/fibonacci256/Map.h

*/
use super::super::focalintent::*;
use super::Pattern;
use smart_leds::hsv::{hsv2rgb, Hsv};
use smart_leds::RGB8;

// Draws rainbows with an ever-changing, widely-varying set of parameters.
// https://gist.github.com/kriegsman/964de772d64c502760e5
// TODO: instead of u16s, use proper types
pub struct Waves {
    pseudotime: u16,
    last_ms: u16,
    hue: u16,
    saturation_bpm: Accum88,
    saturation_min: u16,
    saturation_max: u16,
    bright_depth_bpm: Accum88,
    bright_depth_min: u16,
    bright_depth_max: u16,
    bright_theta_inc_bpm: Accum88,
    bright_theta_inc_min: u16,
    bright_theta_inc_max: u16,
    ms_multiplier_bpm: Accum88,
    ms_multiplier_min: u16,
    ms_multiplier_max: u16,
    hue_inc_bpm: Accum88,
    hue_inc_min: u16,
    hue_inc_max: u16,
    s_hue_bpm: Accum88,
    s_hue_min: u16,
    s_hue_max: u16,
}

impl Waves {
    pub fn new() -> Self {
        Self {
            pseudotime: 0,
            last_ms: 0,
            hue: 0,
            saturation_bpm: 87u16.into(),
            saturation_min: 220,
            saturation_max: 250,
            bright_depth_bpm: 256u16.into(),
            bright_depth_min: 96,
            bright_depth_max: 224,
            bright_theta_inc_bpm: 203u16.into(),
            bright_theta_inc_min: 25 * 256,
            bright_theta_inc_max: 40 * 256,
            ms_multiplier_bpm: 147u16.into(),
            ms_multiplier_min: 23,
            ms_multiplier_max: 60,
            hue_inc_bpm: 113u16.into(),
            hue_inc_min: 1,
            hue_inc_max: 3072,
            s_hue_bpm: 2u16.into(),
            s_hue_min: 5,
            s_hue_max: 9,
        }
    }
}

impl Pattern for Waves {
    /// TODO: this is not correct. write tests for beatsin
    /// TODO: https://github.com/jasoncoon/esp8266-fastled-webserver/blob/fibonacci256/esp8266-fastled-webserver.ino#L1183
    fn buffer(&mut self, now: u32, leds: &mut [RGB8]) {
        let sat8 = beatsin(
            self.saturation_bpm,
            self.saturation_min,
            self.saturation_max,
            now,
            0,
        ) as u8;

        let bright_depth = beatsin(
            self.bright_depth_bpm,
            self.bright_depth_min,
            self.bright_depth_max,
            now,
            0,
        ) as u8;

        let bright_theta_inc = beatsin(
            self.bright_theta_inc_bpm,
            self.bright_theta_inc_min,
            self.bright_theta_inc_max,
            now,
            0,
        );

        let ms_multiplier = beatsin(
            self.ms_multiplier_bpm,
            self.ms_multiplier_min,
            self.ms_multiplier_max,
            now,
            0,
        );

        let mut hue16 = self.hue;

        let hueinc16 = beatsin(self.hue_inc_bpm, self.hue_inc_min, self.hue_inc_max, now, 0);

        let deltams = (now as u16) - self.last_ms;

        self.last_ms = now as u16;

        self.pseudotime += deltams * ms_multiplier;

        self.hue += deltams * beatsin(self.s_hue_bpm, self.s_hue_min, self.s_hue_max, now, 0);

        let mut bright_theta = self.pseudotime;

        for led in leds.iter_mut().rev() {
            // TODO: saturating or wrapping add?
            hue16 += hueinc16;

            // TODO: what are we doing to the hue here? why not just scale16 to get the index?
            let h16_128: u16 = hue16 >> 7;

            let hue8: u8 = if h16_128 & 0x100 != 0 {
                255 - (h16_128 >> 1)
            } else {
                h16_128 >> 1
            } as u8;

            // TODO: saturating or wrapping add?
            bright_theta += bright_theta_inc;

            let b16: u16 = (sin16(bright_theta).wrapping_add(32767).wrapping_add(1)) as u16;

            let bri16 = (((b16 as u32) * (b16 as u32)) / 65536) as u16;

            let mut bri8: u8 = (((bri16 as u32) * (bright_depth as u32)) / 65536) as u8;

            // TODO: saturating or wrapping add?
            bri8 += 255 - bright_depth;

            // TODO: port ColorFromPalette and use it here
            // let index: u8 = scale8(hue8, 240);
            let new_color = Hsv {
                hue: hue8,
                sat: sat8,
                val: bri8,
            };

            let new_color = hsv2rgb(new_color);

            nblend(led, &new_color, 64);
        }
    }
}
