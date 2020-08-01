//! inspired by EVERY_N_MILLIS in https://github.com/FastLED/FastLED/blob/master/lib8tion.h
use core::convert::Infallible;
use core::sync::atomic::{AtomicU32, Ordering};
use nb::block;

/// TODO: figure out how to properly use rtic resources
/// TODO: do we even need an atomic here? would a static mut u32 work fine? i think radio changing it could matter
static ELAPSED_MS: AtomicU32 = AtomicU32::new(0);

/// Keep track of the milliseconds since boot.
/// TODO: function for setting this to a specific amount. We can use the oldest ELAPSED_MS on the network
#[derive(Clone)]
pub struct ElapsedMs;

impl ElapsedMs {
    /// Block for some milliseconds.
    pub fn block(&self, millis: u32) {
        let mut until = EveryNMillis::new(self.clone(), millis);

        until.block();
    }

    /// Increment the time. Call this every millisecond.
    /// This should probably be called inside an interrupt.
    #[inline]
    pub fn increment(&self) {
        self.increment_by(1);
    }

    /// Increment the time by a configurable amount.
    /// thumbv6 doesn't have fetch_add
    /// This shuold probably be called if you ever disable interrupts.
    #[inline]
    #[cfg(feature = "thumbv6")]
    pub fn increment_by(&self, by: u32) {
        cortex_m::interrupt::free(|_cs| {
            let x = ELAPSED_MS.load(Ordering::SeqCst);

            ELAPSED_MS.store(x + by, Ordering::SeqCst);
        });
    }

    /// Increment the time by a configurable amount.
    /// This shuold probably be called if you ever disable interrupts.
    #[inline]
    #[cfg(not(feature = "thumbv6"))]
    pub fn increment_by(&self, by: u32) {
        ELAPSED_MS.fetch_add(by, Ordering::SeqCst);
    }

    /// Load the current time in milliseconds since boot
    #[inline]
    pub fn now(&self) -> u32 {
        ELAPSED_MS.load(Ordering::SeqCst)
    }
}

/// TODO: what should we call this?
/// TODO: is usize a good type? maybe usize?
/// TODO: maybe a custom type so that we can have hz, or ms, or seconds, or minutes, etc.
pub struct EveryNMillis {
    elapsed_ms: ElapsedMs,
    next_trigger: u32,
    period_ms: u32,
}

impl EveryNMillis {
    pub fn new(elapsed_ms: ElapsedMs, period_ms: u32) -> Self {
        Self {
            elapsed_ms,
            next_trigger: 0,
            period_ms,
        }
    }

    pub fn ready(&mut self) -> nb::Result<u32, Infallible> {
        let now = self.now();

        if now < self.next_trigger {
            return Err(nb::Error::WouldBlock);
        }

        // TODO: what if this is late? should we delay less?
        self.next_trigger = now + self.period_ms;

        Ok(now)
    }

    /// Block until this is ready
    pub fn block(&mut self) {
        block!(self.ready()).unwrap();
    }

    pub fn now(&self) -> u32 {
        self.elapsed_ms.now()
    }
}
