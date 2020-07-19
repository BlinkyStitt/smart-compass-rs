use shared_bus::mutex::BusMutex;

// /// do NOT use this. it will fight rtic!
// pub use crate::hal::cortex_m::interrupt::Mutex;

/// This mutex does NOT actually lock anything!
/// Only use this when rtic is already handling safe concurrency
pub struct DummyMutex<T> { inner: T }

impl<T> BusMutex<T> for DummyMutex<T> {
    fn create(inner: T) -> Self {
        Self { inner }
    }

    fn lock<R, F: FnOnce(&T) -> R>(&self, f: F) -> R {
        f(&self.inner)
    }
}
