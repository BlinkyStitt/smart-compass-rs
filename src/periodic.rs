//! inspired by EVERY_N_MILLIS in https://github.com/FastLED/FastLED/blob/master/lib8tion.h

pub struct Periodic {
    previous_trigger: u32,
    period: u32,
}

impl Periodic {
    pub fn new(period: u32) -> Self {
        Self {
            previous_trigger: 0,
            period,
        }
    }

    pub fn ready(&mut self, now: &u32) -> bool {
        let is_ready = now - self.previous_trigger >= self.period;

        if is_ready {
            // TODO: what if this is late?
            self.previous_trigger = *now;
        }

        is_ready
    }
}
