//! The Feather m0 comes with a voltage divider connected to a pin so we can read the voltage.
//! i don't see that on the stm32f3 discovery
//! i think it's fine to just use
use stm32f3_discovery::prelude::*;

use crate::hal::gpio::{gpioc, Input, PullUp};
use crate::periodic::Periodic;

#[derive(Copy, Clone, PartialEq)]
pub enum BatteryStatus {
    Low,
    Ok,
}

pub struct Battery {
    status: BatteryStatus,
    // TODO: the breakout board pulls this high. that means we want pullup here, right? ro does that make the board also do something?
    pin: gpioc::PCx<Input<PullUp>>,
    check_interval: Periodic,
}

impl Battery {
    pub fn new<PE8Mode>(
        pin: gpioc::PC8<PE8Mode>,
        moder: &mut gpioc::MODER,
        pupdr: &mut gpioc::PUPDR,
        check_ms: u32,
    ) -> Self {
        let pin = pin.into_pull_up_input(moder, pupdr).downgrade();

        // TODO: use rtic's periodic tasks instead of our own
        // TODO: should this use the rtc?
        let check_interval = Periodic::new(check_ms);

        Self {
            status: BatteryStatus::Ok,
            pin,
            check_interval,
        }
    }

    pub fn check(&mut self) -> (bool, BatteryStatus) {
        let mut changed = false;

        if self.check_interval.ready() {
            let new_status = if self.pin.is_high().unwrap() {
                BatteryStatus::Ok
            } else {
                BatteryStatus::Low
            };

            if self.status != new_status {
                self.status = new_status;
                changed = true;
            }
        }

        return (changed, self.status);
    }
}
