use super::super::focalintent::*;
use super::{ANGLES, FIBONACCI_TO_PHYSICAL, PHYSICAL_TO_FIBONACCI};
use crate::timers::ElapsedMs;
use core::cmp::Ordering;
use smart_leds::{colors, RGB8};

pub struct Clock {
    hour_angle: u8,
    minute_angle: u8,
    second_angle: u8,
    background_fade: u8,
}

impl Clock {
    pub fn new(background_fade: u8) -> Self {
        assert!(background_fade > 0);

        Self {
            hour_angle: 0,
            minute_angle: 0,
            second_angle: 0,
            background_fade,
        }
    }

    pub fn buffer(&mut self, elapsed_ms: &ElapsedMs, leds: &mut [RGB8], time: &time::Time) {
        let second = time.second() as f32;

        // float minute = timeClient.getMinutes() + (second / 60.0);
        let minute = (time.minute() as f32) + (second / 60.0);

        // float hour = timeClient.getHours() + (minute / 60.0);
        let hour = (time.hour() as f32) + (minute / 60.0);

        // TODO: move these
        const HOUR_RADIUS: u8 = 96;
        const MINUTE_RADIUS: u8 = 178;
        const SECOND_RADIUS: u8 = 255;
        const HAND_WIDTH: u8 = 8;

        const DEGREES_PER_SECOND: f32 = 255.0 / 60.0;
        const DEGREES_PER_MINUTE: f32 = 255.0 / 60.0;
        const DEGREES_PER_HOUR: f32 = 255.0 / 12.0;

        // TODO: do this every 100 ms
        self.hour_angle = 255 - (hour * DEGREES_PER_HOUR) as u8;
        self.minute_angle = 255 - (minute * DEGREES_PER_MINUTE) as u8;
        self.second_angle = 255 - (second * DEGREES_PER_SECOND) as u8;

        fade_to_black_by(leds, self.background_fade);

        antialias_pixel_ar(
            leds,
            self.second_angle,
            HAND_WIDTH,
            0,
            SECOND_RADIUS,
            colors::BLUE,
        );
        antialias_pixel_ar(
            leds,
            self.minute_angle,
            HAND_WIDTH,
            0,
            MINUTE_RADIUS,
            colors::GREEN,
        );
        antialias_pixel_ar(
            leds,
            self.hour_angle,
            HAND_WIDTH,
            0,
            HOUR_RADIUS,
            colors::RED,
        );

        leds[0] = colors::RED;
    }
}

pub fn draw_spiral_line(leds: &mut [RGB8], angle: u8, step: u8, color: &RGB8) {
    let mut start_index = 0;

    let num_leds = leds.len();

    let mut smallest_angle_difference = 255;

    // find the outermost led closest to the desired angle
    // for (int i = 0; i < NUM_LEDS; i++) {
    for i in 0..num_leds {
        // int j = physicalToFibonacci[i];
        let j = PHYSICAL_TO_FIBONACCI[i];

        // if (j < step) continue;
        if j < step {
            continue;
        }

        // if (!(j + step >= NUM_LEDS)) continue; // not outermost
        // TODO: i think this can be written differently
        if !(j as usize + step as usize >= num_leds) {
            continue;
        }

        // uint8_t a = angles[i];
        let a = ANGLES[i];

        // if (a == angle) startIndex = i;
        // else if (angle - a > 0 && angle - a < smallestAngleDifference) {
        // smallestAngleDifference = angle - a;
        // startIndex = i;
        // }
        if a == angle {
            start_index = i
        } else if angle - a > 0 && angle - a < smallest_angle_difference {
            smallest_angle_difference = angle - a;
            start_index = i;
        }
    }

    // draw the starting LED
    // TODO: nblend?
    leds[start_index] = *color;

    // draw to center from outer start
    // int f = physicalToFibonacci[startIndex];
    let mut f = PHYSICAL_TO_FIBONACCI[start_index];
    // while (f - step >= 0 && f - step < NUM_LEDS) {
    // TODO: i don't think this handles saturating/overflow correctly
    while (f >= step) && (f - step < leds.len() as u8) {
        // leds[fibonacciToPhysical[f]] += color;
        let index = FIBONACCI_TO_PHYSICAL[f as usize] as usize;

        // TODO: nblend?
        leds[index] = *color;

        f = f - step;
    }
}

// TODO: test this. i'm pretty sure it is wrong
pub fn antialias_pixel_ar(
    leds: &mut [RGB8],
    angle: u8,
    d_angle: u8,
    start_radius: u8,
    end_radius: u8,
    color: RGB8,
) {
    if d_angle == 0 {
        return;
    }

    // TODO: unsure about saturating vs wrapping. this is broken now

    // uint16_t amax = qadd8(angle, dAngle);
    let amax: u8 = angle.wrapping_add(d_angle);
    // uint8_t amin = qsub8(angle, dAngle);
    let amin: u8 = angle.wrapping_sub(d_angle);

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
        // TODO: i'm not sure this is correct
        fade_to_black_by(&mut [color], fade);

        // if (ao <= amax && ao >= amin) {
        if ao <= amax && ao >= amin {
            // uint8_t ro = physicalToFibonacci[o];
            let ro: u8 = PHYSICAL_TO_FIBONACCI[i];

            // if (ro <= endRadius && ro >= startRadius) {
            if ro <= end_radius && ro >= start_radius {
                // leds[i] += faded;
                leds[i] = color;
            }
        }
    }
}
