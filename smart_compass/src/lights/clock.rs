use super::focalintent::*;
use super::patterns;
use smart_leds::{colors, RGB8};

#[derive(Default)]
pub struct AnalogClock {
    hour_angle: u8,
    minute_angle: u8,
    second_angle: u8,
    background_fade: u8,
}

impl AnalogClock {
    pub fn new(background_fade: u8) -> Self {
        assert!(background_fade > 0);

        Self {
            background_fade,
            ..Default::default()
        }
    }

    pub fn drawAnalogClock(
        &mut self,
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
        patterns::antialias_pixel_ar(
            leds,
            self.second_angle,
            HAND_WIDTH,
            0,
            SECOND_RADIUS,
            colors::BLUE,
        );
        // antialiasPixelAR(minuteAngle, handWidth, 0, minuteRadius, CRGB::Green);
        patterns::antialias_pixel_ar(
            leds,
            self.minute_angle,
            HAND_WIDTH,
            0,
            MINUTE_RADIUS,
            colors::GREEN,
        );
        // antialiasPixelAR(hourAngle, handWidth, 0, hourRadius, CRGB::Red);
        patterns::antialias_pixel_ar(
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
