/*
shifting

let shift_n = some number calculated from the elapsed time or the number of frames drawn

light_data.iter().cycle().skip(shift_n).take(256).cloned()

TODO: use https://docs.rs/microfft/0.3.0/microfft/ for sound reactive patterns?

// TODO: port these https://github.com/jasoncoon/esp8266-fastled-webserver/blob/fibonacci256/Map.h

*/
use super::focalintent::*;
use super::NUM_LEDS;
use crate::ELAPSED_MS;
use core::cmp::Ordering;
use smart_leds::hsv::{hsv2rgb, Hsv};
use smart_leds::RGB8;

const PHYSICAL_TO_FIBONACCI: [u8; NUM_LEDS] = [
    0, 13, 26, 39, 52, 65, 78, 91, 104, 117, 130, 143, 156, 169, 182, 195, 208, 221, 234, 247, 252,
    239, 226, 213, 200, 187, 174, 161, 148, 135, 122, 109, 96, 83, 70, 57, 44, 31, 18, 5, 10, 23,
    36, 49, 62, 75, 88, 101, 114, 127, 140, 153, 166, 179, 192, 205, 218, 231, 244, 249, 236, 223,
    210, 197, 184, 171, 158, 145, 132, 119, 106, 93, 80, 67, 54, 41, 28, 15, 2, 7, 20, 33, 46, 59,
    72, 85, 98, 111, 124, 137, 150, 163, 176, 189, 202, 215, 228, 241, 254, 246, 233, 220, 207,
    194, 181, 168, 155, 142, 129, 116, 103, 90, 77, 64, 51, 38, 25, 12, 4, 17, 30, 43, 56, 69, 82,
    95, 108, 121, 134, 147, 160, 173, 186, 199, 212, 225, 238, 251, 243, 230, 217, 204, 191, 178,
    165, 152, 139, 126, 113, 100, 87, 74, 61, 48, 35, 22, 9, 1, 14, 27, 40, 53, 66, 79, 92, 105,
    118, 131, 144, 157, 170, 183, 196, 209, 222, 235, 248, 253, 240, 227, 214, 201, 188, 175, 162,
    149, 136, 123, 110, 97, 84, 71, 58, 45, 32, 19, 6, 11, 24, 37, 50, 63, 76, 89, 102, 115, 128,
    141, 154, 167, 180, 193, 206, 219, 232, 245, 250, 237, 224, 211, 198, 185, 172, 159, 146, 133,
    120, 107, 94, 81, 68, 55, 42, 29, 16, 3, 8, 21, 34, 47, 60, 73, 86, 99, 112, 125, 138, 151,
    164, 177, 190, 203, 216, 229, 242, 255,
];
const FIBONACCI_TO_PHYSICAL: [u8; NUM_LEDS] = [
    0, 157, 78, 235, 118, 39, 196, 79, 236, 156, 40, 197, 117, 1, 158, 77, 234, 119, 38, 195, 80,
    237, 155, 41, 198, 116, 2, 159, 76, 233, 120, 37, 194, 81, 238, 154, 42, 199, 115, 3, 160, 75,
    232, 121, 36, 193, 82, 239, 153, 43, 200, 114, 4, 161, 74, 231, 122, 35, 192, 83, 240, 152, 44,
    201, 113, 5, 162, 73, 230, 123, 34, 191, 84, 241, 151, 45, 202, 112, 6, 163, 72, 229, 124, 33,
    190, 85, 242, 150, 46, 203, 111, 7, 164, 71, 228, 125, 32, 189, 86, 243, 149, 47, 204, 110, 8,
    165, 70, 227, 126, 31, 188, 87, 244, 148, 48, 205, 109, 9, 166, 69, 226, 127, 30, 187, 88, 245,
    147, 49, 206, 108, 10, 167, 68, 225, 128, 29, 186, 89, 246, 146, 50, 207, 107, 11, 168, 67,
    224, 129, 28, 185, 90, 247, 145, 51, 208, 106, 12, 169, 66, 223, 130, 27, 184, 91, 248, 144,
    52, 209, 105, 13, 170, 65, 222, 131, 26, 183, 92, 249, 143, 53, 210, 104, 14, 171, 64, 221,
    132, 25, 182, 93, 250, 142, 54, 211, 103, 15, 172, 63, 220, 133, 24, 181, 94, 251, 141, 55,
    212, 102, 16, 173, 62, 219, 134, 23, 180, 95, 252, 140, 56, 213, 101, 17, 174, 61, 218, 135,
    22, 179, 96, 253, 139, 57, 214, 100, 18, 175, 60, 217, 136, 21, 178, 97, 254, 138, 58, 215, 99,
    19, 176, 59, 216, 137, 20, 177, 98, 255,
];
const COORDS_X: [u8; NUM_LEDS] = [
    133, 156, 165, 168, 165, 158, 147, 132, 114, 95, 76, 57, 41, 28, 19, 15, 17, 24, 37, 56, 123,
    96, 73, 53, 38, 28, 24, 25, 31, 41, 55, 71, 89, 106, 122, 136, 146, 152, 152, 143, 138, 136,
    128, 115, 101, 85, 70, 56, 44, 37, 33, 34, 41, 53, 69, 90, 114, 140, 167, 226, 204, 180, 154,
    129, 106, 85, 67, 54, 46, 43, 44, 50, 60, 72, 86, 100, 113, 123, 128, 117, 104, 90, 78, 67, 59,
    54, 54, 59, 68, 82, 100, 121, 143, 167, 191, 212, 231, 246, 255, 251, 251, 245, 233, 218, 199,
    178, 156, 134, 114, 96, 82, 73, 67, 66, 70, 78, 89, 103, 111, 94, 84, 80, 81, 86, 96, 109, 126,
    145, 165, 185, 204, 220, 233, 241, 244, 241, 232, 217, 179, 201, 217, 229, 235, 235, 230, 220,
    207, 190, 172, 154, 136, 121, 108, 99, 95, 96, 104, 120, 110, 111, 118, 130, 144, 160, 176,
    192, 206, 217, 224, 227, 224, 216, 202, 184, 162, 137, 110, 44, 68, 94, 120, 145, 168, 187,
    202, 212, 216, 216, 212, 203, 191, 177, 162, 148, 135, 126, 122, 136, 147, 161, 174, 186, 197,
    204, 206, 205, 198, 187, 172, 152, 130, 106, 81, 58, 36, 17, 0, 5, 15, 30, 49, 71, 93, 116,
    138, 157, 173, 185, 192, 195, 193, 187, 178, 166, 152, 137, 149, 164, 175, 180, 182, 179, 171,
    159, 143, 125, 105, 83, 63, 44, 28, 16, 9, 7, 12, 23,
];
const COORDS_Y: [u8; NUM_LEDS] = [
    126, 120, 109, 96, 82, 69, 57, 49, 45, 45, 50, 59, 74, 92, 114, 138, 163, 188, 211, 231, 255,
    248, 235, 218, 198, 175, 152, 129, 107, 89, 74, 63, 57, 56, 59, 66, 76, 88, 102, 116, 103, 88,
    77, 71, 68, 70, 77, 88, 103, 121, 141, 163, 184, 205, 222, 236, 245, 249, 247, 208, 224, 235,
    241, 240, 234, 223, 209, 191, 172, 152, 132, 115, 101, 90, 84, 82, 86, 95, 114, 107, 98, 98,
    103, 112, 126, 142, 159, 177, 195, 210, 222, 230, 233, 230, 223, 209, 191, 168, 142, 98, 125,
    151, 174, 194, 209, 219, 223, 223, 218, 208, 195, 180, 164, 148, 134, 122, 114, 112, 123, 128,
    138, 151, 165, 180, 193, 203, 211, 214, 212, 206, 194, 178, 158, 134, 109, 83, 58, 35, 11, 28,
    48, 71, 95, 120, 142, 163, 179, 192, 200, 203, 202, 196, 187, 175, 162, 148, 136, 133, 152,
    166, 177, 186, 190, 191, 187, 178, 165, 148, 128, 107, 84, 62, 41, 24, 11, 2, 0, 28, 16, 9, 8,
    13, 23, 37, 55, 75, 96, 116, 135, 151, 164, 173, 177, 177, 172, 162, 146, 153, 161, 163, 160,
    152, 139, 124, 106, 87, 69, 51, 36, 25, 18, 16, 20, 29, 44, 64, 133, 106, 81, 60, 44, 32, 26,
    25, 29, 38, 50, 65, 82, 99, 115, 129, 140, 147, 148, 138, 134, 131, 122, 110, 95, 80, 65, 52,
    42, 36, 34, 37, 45, 59, 77, 98, 123, 149, 176, 202,
];
const ANGLES: [u8; NUM_LEDS] = [
    0, 247, 238, 229, 220, 211, 203, 194, 185, 176, 167, 159, 150, 141, 132, 123, 115, 106, 97, 88,
    65, 74, 83, 92, 100, 109, 118, 127, 136, 144, 153, 162, 171, 180, 188, 197, 206, 215, 224, 232,
    209, 201, 192, 183, 174, 165, 157, 148, 139, 130, 121, 113, 104, 95, 86, 77, 69, 60, 51, 28,
    37, 46, 54, 63, 72, 81, 90, 98, 107, 116, 125, 134, 142, 151, 160, 169, 178, 186, 195, 172,
    163, 155, 146, 137, 128, 119, 111, 102, 93, 84, 75, 67, 58, 49, 40, 31, 23, 14, 5, 246, 255, 8,
    17, 26, 35, 44, 52, 61, 70, 79, 88, 96, 105, 114, 123, 132, 140, 149, 135, 126, 117, 108, 100,
    91, 82, 73, 64, 56, 47, 38, 29, 20, 12, 3, 250, 241, 232, 223, 209, 218, 227, 235, 244, 253, 6,
    15, 24, 33, 41, 50, 59, 68, 77, 85, 94, 103, 112, 98, 89, 80, 71, 62, 54, 45, 36, 27, 18, 10,
    1, 247, 239, 230, 221, 212, 203, 195, 186, 163, 172, 180, 189, 198, 207, 216, 224, 233, 242,
    251, 4, 13, 22, 31, 39, 48, 57, 66, 75, 52, 43, 34, 25, 16, 8, 254, 245, 237, 228, 219, 210,
    201, 193, 184, 175, 166, 157, 149, 126, 134, 143, 152, 161, 170, 178, 187, 196, 205, 214, 222,
    231, 240, 249, 2, 11, 20, 28, 37, 14, 5, 252, 243, 235, 226, 217, 208, 199, 191, 182, 173, 164,
    155, 147, 138, 129, 120, 111, 103,
];

pub trait Pattern {
    fn draw(&mut self, leds: &mut [RGB8]);
}

// Draws rainbows with an ever-changing, widely-varying set of parameters.
// https://gist.github.com/kriegsman/964de772d64c502760e5
#[derive(Default)]
pub struct Pride {
    pseudotime: u16,
    last_ms: u16,
    hue: u16,
}

impl Pattern for Pride {
    /// TODO: should we just call this draw? should Pride have a Pattern trait?
    /// TODO: https://github.com/jasoncoon/esp8266-fastled-webserver/blob/fibonacci256/esp8266-fastled-webserver.ino#L1183
    fn draw(&mut self, leds: &mut [RGB8]) {
        let now = unsafe { ELAPSED_MS };

        // TODO: figure out what all these numbers do and make it look good on two concentric rings
        // uint8_t sat8 = beatsin88(87, 220, 250);
        let sat8 = beatsin88(87, 220, 250, now, 0) as u8;
        // uint8_t brightdepth = beatsin88(341, 96, 224);
        let brightdepth = beatsin88(341, 96, 224, now, 0) as u8;
        // uint16_t brightnessthetainc16 = beatsin88(203, (25 * 256), (40 * 256));
        let brightnessthetainc16 = beatsin88(203, 25 * 256, 40 * 256, now, 0);
        // uint8_t msmultiplier = beatsin88(147, 23, 60);
        let msmultiplier = beatsin88(147, 23, 60, now, 0);

        // uint16_t hue16 = sHue16;
        let mut hue16 = self.hue;
        // uint16_t hueinc16 = beatsin88(113, 1, 3000);
        let hueinc16 = beatsin88(113, 1, 3000, now, 0);

        // uint16_t ms = network_ms; // this should keep everyone's lights looking the same
        // uint16_t deltams = ms - sLastMillis;
        let deltams = now as u16 - self.last_ms;

        // sLastMillis = ms;
        self.last_ms = now as u16;

        // sPseudotime += deltams * msmultiplier;
        self.pseudotime += deltams * msmultiplier;

        // sHue16 += deltams * beatsin88(400, 5, 9);
        self.hue += deltams * beatsin88(400, 5, 9, now, 0);

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

enum Direction {
    In,
    Out,
}

pub struct Wheel {
    // hue: u8,
    direction: Direction,
    pixel_map: &'static [u8],
}

impl Wheel {
    pub fn new() -> Self {
        Self {
            // hue: 0,
            direction: Direction::Out,
            pixel_map: &PHYSICAL_TO_FIBONACCI,
        }
    }
}

impl Pattern for Wheel {
    /// TODO: fastled did something special for rainbows. do the same here
    fn draw(&mut self, leds: &mut [RGB8]) {
        // divide to slow down the animation.
        // TODO: or we could advance by 1 per frame
        let now = unsafe { ELAPSED_MS } / 30;

        for (i, led) in leds.iter_mut().enumerate() {
            let hue = match self.direction {
                Direction::In => now + (self.pixel_map[i] as u32 * 3) / 7,
                Direction::Out => now - (self.pixel_map[i] as u32 * 3) / 7,
            } as u8;

            let new_color = hsv2rgb(Hsv {
                hue,
                sat: 240,
                val: 255,
            });

            // TODO: is there a better way to do this?
            led.r = new_color.r;
            led.g = new_color.g;
            led.b = new_color.b;
        }
    }
}

/*
/// Input a value 0 to 255 to get a color value
/// The colours are a transition r - g - b - back to r.
/// TODO: use HSV instead? https://github.com/jasoncoon/esp8266-fastled-webserver/blob/fibonacci256/esp8266-fastled-webserver.ino#L1409
pub fn wheel(mut wheel_pos: u8) -> RGB8 {
    wheel_pos = 255 - wheel_pos;
    if wheel_pos < 85 {
        return (255 - wheel_pos * 3, 0, wheel_pos * 3).into();
    }
    if wheel_pos < 170 {
        wheel_pos -= 85;
        return (0, wheel_pos * 3, 255 - wheel_pos * 3).into();
    }
    wheel_pos -= 170;
    (wheel_pos * 3, 255 - wheel_pos * 3, 0).into()
}
*/

// TODO: test this
pub fn antialias_pixel_ar(
    leds: &mut [RGB8],
    angle: u8,
    d_angle: u8,
    start_radius: u8,
    end_radius: u8,
    mut color: RGB8,
) {
    // uint16_t amax = qadd8(angle, dAngle);
    let amax: u8 = angle.saturating_add(d_angle);
    // uint8_t amin = qsub8(angle, dAngle);
    let amin: u8 = angle.saturating_sub(d_angle);

    // for (uint16_t i = 0; i < NUM_LEDS; i++) {
    for i in 0..leds.len() {
        // uint8_t o = i;

        // uint8_t ao = angles[o];
        let ao: u8 = ANGLES[i];

        // uint8_t adiff = qsub8(max(ao, angle), min(ao, angle));
        // let adiff = max(ao, angle).saturating_sub(min(ao, angle));
        let adiff = match ao.cmp(&angle) {
            Ordering::Less => angle - ao,
            Ordering::Greater => ao - angle,
            Ordering::Equal => 0,
        };

        // uint8_t fade = qmul8(adiff, 32);
        let fade: u8 = adiff.saturating_mul(32);

        // CRGB faded = color;
        // faded.fadeToBlackBy(fade);
        fade_to_black_by(&mut [color], fade);

        // if (ao <= amax && ao >= amin) {
        if ao <= amax && ao >= amin {
            // uint8_t ro = physicalToFibonacci[o];
            let ro: u8 = PHYSICAL_TO_FIBONACCI[i];

            // if (ro <= endRadius && ro >= startRadius) {
            if ro <= end_radius && ro >= start_radius {
                // leds[i] += faded;
                leds[i] += color;
            }
        }
    }
}
