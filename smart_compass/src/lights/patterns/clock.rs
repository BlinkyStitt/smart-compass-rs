use super::super::focalintent::*;
use super::{ANGLES, PHYSICAL_TO_FIBONACCI};
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

    pub fn buffer(
        &mut self,
        elapsed_ms: &ElapsedMs,
        leds: &mut [RGB8],
        mut hour: f32,
        mut minute: f32,
        second: f32,
    ) {
        // float second = timeClient.getSeconds();

        // float minute = timeClient.getMinutes() + (second / 60.0);
        minute += second / 60.0;

        // float hour = timeClient.getHours() + (minute / 60.0);
        hour += minute / 60.0;

        // static uint8_t hourAngle = 0;
        // static uint8_t minuteAngle = 0;
        // static uint8_t secondAngle = 0;

        // const uint8_t hourRadius = 96;
        const HOUR_RADIUS: u8 = 96;
        // const uint8_t minuteRadius = 192;
        const MINUTE_RADIUS: u8 = 192;
        // const uint8_t secondRadius = 255;
        const SECOND_RADIUS: u8 = 255;

        // const uint8_t handWidth = 32;
        const HAND_WIDTH: u8 = 32;

        // const float degreesPerSecond = 255.0 / 60.0;
        const DEGREES_PER_SECOND: f32 = 255.0 / 60.0;
        // const float degreesPerMinute = 255.0 / 60.0;
        const DEGREES_PER_MINUTE: f32 = 255.0 / 60.0;
        // const float degreesPerHour = 255.0 / 12.0;
        const DEGREES_PER_HOUR: f32 = 255.0 / 12.0;

        // EVERY_N_MILLIS(100) {
        //   hourAngle = 255 - hour * degreesPerHour;
        //   minuteAngle = 255 - minute * degreesPerMinute;
        //   secondAngle = 255 - second * degreesPerSecond;
        // }
        // TODO: do this every 100 ms
        self.hour_angle = 255 - (hour * DEGREES_PER_HOUR) as u8;
        self.minute_angle = 255 - (minute * DEGREES_PER_MINUTE) as u8;
        self.second_angle = 255 - (second * DEGREES_PER_SECOND) as u8;

        // fadeToBlackBy(leds, NUM_LEDS, clockBackgroundFade);
        fade_to_black_by(leds, self.background_fade);

        // antialiasPixelAR(secondAngle, handWidth, 0, secondRadius, CRGB::Blue);
        antialias_pixel_ar(
            leds,
            self.second_angle,
            HAND_WIDTH,
            0,
            SECOND_RADIUS,
            colors::BLUE,
        );
        // antialiasPixelAR(minuteAngle, handWidth, 0, minuteRadius, CRGB::Green);
        antialias_pixel_ar(
            leds,
            self.minute_angle,
            HAND_WIDTH,
            0,
            MINUTE_RADIUS,
            colors::GREEN,
        );
        // antialiasPixelAR(hourAngle, handWidth, 0, hourRadius, CRGB::Red);
        antialias_pixel_ar(
            leds,
            self.hour_angle,
            HAND_WIDTH,
            0,
            HOUR_RADIUS,
            colors::RED,
        );

        // leds[0] = CRGB::Red;
        leds[0] = colors::RED;
    }
}

// TODO: test this
pub fn antialias_pixel_ar(
    leds: &mut [RGB8],
    angle: u8,
    d_angle: u8,
    start_radius: u8,
    end_radius: u8,
    color: RGB8,
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
        // TODO: i'm not sure this is correct
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
