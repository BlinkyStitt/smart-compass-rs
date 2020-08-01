//! The Feather m0 comes with a voltage divider connected to a pin so we can read the voltage.
//! i don't see that on the stm32f3 discovery
//! i think it's fine to just use
use crate::timers::{ElapsedMs, EveryNMillis};
use embedded_hal::digital::v2::InputPin;

#[derive(Copy, Clone, PartialEq)]
pub enum BatteryStatus {
    Low,
    Ok,
}

pub struct Battery<StatusInputPin> {
    status: BatteryStatus,
    pin: StatusInputPin,
    check_interval: EveryNMillis,
}

impl<StatusInputPin: InputPin> Battery<StatusInputPin> {
    pub fn new(pin: StatusInputPin, elapsed_ms: &ElapsedMs, check_ms: u32) -> Self {
        // TODO: use rtic's periodic tasks instead of our own
        let check_interval = EveryNMillis::new(elapsed_ms, check_ms);

        Self {
            status: BatteryStatus::Ok,
            pin,
            check_interval,
        }
    }

    pub fn check(&mut self, elapsed_ms: &ElapsedMs) -> (bool, BatteryStatus) {
        let mut changed = false;

        if let Ok(_) = self.check_interval.ready(elapsed_ms) {
            let new_status = if self.pin.is_high().ok().unwrap() {
                BatteryStatus::Ok
            } else {
                BatteryStatus::Low
            };

            if self.status != new_status {
                self.status = new_status;
                changed = true;
            }
        }

        (changed, self.status)
    }
}
