//! inspired by EVERY_N_MILLIS in https://github.com/FastLED/FastLED/blob/master/lib8tion.h

/// TODO: what should we call this?
/// TODO: is usize a good type? maybe usize?
/// TODO: maybe a custom type so that we can have hz, or ms, or seconds, or minutes, etc.
pub struct Periodic {
    previous_trigger: u32,
    period_ms: u32,
}

impl Periodic {
    pub fn new(period_ms: u32) -> Self {
        Self {
            previous_trigger: 0,
            period_ms,
        }
    }

    pub fn ready(&mut self) -> bool {
        let now = unsafe { crate::ELAPSED_MS };

        let is_ready = now - self.previous_trigger >= self.period_ms;

        if is_ready {
            // TODO: what if this is late?
            self.previous_trigger = now;
        }

        is_ready
    }
}
