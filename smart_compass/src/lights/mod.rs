mod focalintent;
mod patterns;

use self::patterns::Pattern;
use crate::location::GpsData;
use crate::network::NetworkData;
use crate::timers::{ElapsedMs, EveryNMillis};
use crate::NUM_LEDS;
use accelerometer::Orientation;
use embedded_hal::digital::v2::OutputPin;
use smart_leds::{brightness, gamma, SmartLedsWrite, RGB8};

/// TODO: better trait bounds?
pub struct Lights<SmartLeds: SmartLedsWrite> {
    pub brightness: u8,
    pub framerate: EveryNMillis,

    last_orientation: Orientation,
    leds: SmartLeds,

    // TODO: use a Vec?
    led_buffer: [RGB8; NUM_LEDS],

    // TODO: think about this more
    pattern_clock: patterns::Clock,
    pattern_pacman: patterns::PacMan,
    pattern_pride: patterns::Pride,
    pattern_sunflower: patterns::Sunflower,
    pattern_test_map: patterns::TestMap,
    pattern_waves: patterns::Waves,
    pattern_compass: patterns::Compass,
}

impl<SmartLeds: SmartLedsWrite> Lights<SmartLeds>
where
    SmartLeds::Color: core::convert::From<smart_leds::RGB<u8>>,
{
    pub fn new(
        leds: SmartLeds,
        brightness: u8,
        elapsed_ms: &ElapsedMs,
        my_peer_id: Option<usize>,
        frames_per_second: u8,
    ) -> Self {
        let light_data: [RGB8; NUM_LEDS] = [RGB8::default(); NUM_LEDS];

        let framerate_ms = 1_000 / (frames_per_second as u32);

        let framerate = EveryNMillis::new(elapsed_ms, framerate_ms);

        let last_orientation = Orientation::Unknown;

        let pattern_compass = patterns::Compass::new(3000.0);
        let pattern_clock = patterns::Clock::new(240);
        let pattern_pacman = patterns::PacMan::new();
        let pattern_pride = patterns::Pride::new();
        let pattern_sunflower = patterns::Sunflower::new();
        let pattern_test_map = patterns::TestMap::new();
        let pattern_waves = patterns::Waves::new();

        Self {
            brightness,
            framerate,
            last_orientation,
            led_buffer: light_data,
            leds,
            pattern_compass,
            pattern_clock,
            pattern_pacman,
            pattern_pride,
            pattern_sunflower,
            pattern_test_map,
            pattern_waves,
        }
    }

    // fill the buffer with the light data
    fn _buffer(
        &mut self,
        elapsed_ms: &ElapsedMs,
        orientation: &Orientation,
        gps_data: Option<&GpsData>,
        network_data: Option<&NetworkData>,
    ) {
        // TODO: match or something to pick between a bunch of different patterns
        let orientation_changed = self.last_orientation == *orientation;

        // TODO: have a Pattern state machine that handles orientation and transitionary animations
        match orientation {
            Orientation::FaceDown => {
                // render flashlight
                todo!("flashlight pattern");
            }
            Orientation::FaceUp => {
                // TODO: DEBUGGING! MOVE `Orientation::Unknown` to where it belongs
                // render compass
                let now = elapsed_ms.now();

                // TODO: if no peer locations, draw the loading pattern

                let network_data = network_data.unwrap();

                if network_data.my_peer_id != 0 {
                    self.pattern_compass.buffer(
                        now,
                        &mut self.led_buffer,
                        gps_data.unwrap(),
                        network_data,
                    );
                } else {
                    // TODO: pattern for loading
                    self.pattern_sunflower.buffer(now, &mut self.led_buffer);
                }
            }
            Orientation::PortraitDown | Orientation::Unknown => {
                let gps_data = gps_data.unwrap();

                // render clock
                if let Some(time) = &gps_data.time {
                    self.pattern_clock
                        .buffer(elapsed_ms, &mut self.led_buffer, time);
                } else {
                    let now = elapsed_ms.now();

                    // TODO: pattern for loading
                    self.pattern_sunflower.buffer(now, &mut self.led_buffer);
                }
            }
            Orientation::LandscapeUp | Orientation::LandscapeDown | Orientation::PortraitUp => {
                // render pretty lights
                // TODO: lots of different patterns to choose from
                let now = elapsed_ms.now();
                /*
                let now = ELAPSED_MS.now();

                // TODO: how should we scale now?
                // let j = self.frames_drawn % (NUM_LEDS as u32 * 5);
                // TODO: why multiply by 5?
                let j = (now / 3) % (NUM_LEDS as u32 * 5);

                for i in 0..NUM_LEDS {
                    self.led_buffer[i] = patterns::wheel(
                        (((i * 256) as u16 / NUM_LEDS as u16 + j as u16) & 255) as u8,
                    );
                }
                */
                // self.pattern_sunflower.buffer(now, &mut self.led_buffer);
                // self.pattern_pride.buffer(now, &mut self.led_buffer);
                // self.pattern_waves.buffer(now, &mut self.led_buffer);
                // self.pattern_test_map.buffer(now, &mut self.led_buffer);
                self.pattern_pacman.buffer(now, &mut self.led_buffer);
            }
        };

        if orientation_changed {
            self.last_orientation = *orientation;
        }
    }

    #[cfg(feature = "lights_interrupt_free")]
    #[inline(always)]
    fn _draw(&mut self, elapsed_ms: &ElapsedMs) -> u32 {
        let data = self.led_buffer;

        // correct the colors
        // TODO: do we really need cloned here?
        let data = gamma(data.iter().cloned());

        // dim the lights
        let data = brightness(data, self.brightness);

        let start = elapsed_ms.now();

        // disable interrupts
        cortex_m::interrupt::free(|_| {
            // display
            self.leds.write(data).ok().unwrap();

            // TODO: from a quick test, it looks like drawing 256 WS2812 takes 12-13ms
            // TODO: this should probably be configurable
            elapsed_ms.increment_by(10);
        });

        elapsed_ms.now() - start
    }

    #[cfg(not(feature = "lights_interrupt_free"))]
    #[inline(always)]
    fn _draw(&mut self, elapsed_ms: &ElapsedMs) -> u32 {
        let data = self.led_buffer.clone();

        // correct the colors
        // TODO: do we really need cloned here?
        let data = gamma(data.iter().cloned());

        // dim the lights
        let data = brightness(data, self.brightness);

        let start = elapsed_ms.now();

        // display
        self.leds.write(data).ok().unwrap();

        elapsed_ms.now() - start
    }

    /// TODO: should this fill the buffer with black, too?
    /// TODO: I think FastLED had helpers to do this quickly
    pub fn draw_black(&mut self, elapsed_ms: &ElapsedMs) {
        focalintent::fade_to_black_by(&mut self.led_buffer, 255);

        self._draw(elapsed_ms);
    }

    pub fn draw_test_pattern(&mut self, elapsed_ms: &ElapsedMs) {
        // 1 red
        self.led_buffer[0].r = 0xFF;

        // 2 green
        self.led_buffer[1].g = 0xFF;
        self.led_buffer[2].g = 0xFF;

        // 3 blue
        self.led_buffer[3].b = 0xFF;
        self.led_buffer[4].b = 0xFF;
        self.led_buffer[5].b = 0xFF;

        // 4 white spread out
        self.led_buffer[6].r = 0xFF;
        self.led_buffer[6].g = 0xFF;
        self.led_buffer[6].b = 0xFF;

        self.led_buffer[8].r = 0xFF;
        self.led_buffer[8].g = 0xFF;
        self.led_buffer[8].b = 0xFF;

        self.led_buffer[10].r = 0xFF;
        self.led_buffer[10].g = 0xFF;
        self.led_buffer[10].b = 0xFF;

        self.led_buffer[255].r = 0xFF;
        self.led_buffer[255].g = 0xFF;
        self.led_buffer[255].b = 0xFF;

        self._draw(elapsed_ms);
    }

    pub fn draw(
        &mut self,
        elapsed_ms: &ElapsedMs,
        gps: Option<&GpsData>,
        network: Option<&NetworkData>,
        orientation: &Orientation,
    ) -> Option<(u32, u32, u32)> {
        let start = self.framerate.ready(elapsed_ms).ok()?;

        // TODO: warn if framerate is too fast for us to keep up. will need to keep track of the last time we drew

        // fill the light buffer
        // TODO: make it possible to call buffer seperate from draw
        self._buffer(elapsed_ms, orientation, gps, network);

        // display
        // TODO! some drivers disable interrupts while they draw! this means we won't have an accurate ELAPSED_MS!
        let draw_time = self._draw(elapsed_ms);

        let total_time = elapsed_ms.now() - start;

        // TODO: calculate actual framerate

        Some((start, draw_time, total_time))
    }
}
