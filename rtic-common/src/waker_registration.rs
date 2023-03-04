//! Waker registration utility.

use core::cell::UnsafeCell;
use core::task::Waker;

/// A critical section based waker handler.
pub struct CriticalSectionWakerRegistration {
    waker: UnsafeCell<Option<Waker>>,
}

unsafe impl Send for CriticalSectionWakerRegistration {}
unsafe impl Sync for CriticalSectionWakerRegistration {}

impl CriticalSectionWakerRegistration {
    /// Create a new waker registration.
    pub const fn new() -> Self {
        Self {
            waker: UnsafeCell::new(None),
        }
    }

    /// Register a waker.
    /// This will overwrite the previous waker if there was one.
    pub fn register(&self, new_waker: &Waker) {
        critical_section::with(|_| {
            // SAFETY: This access is protected by the critical section.
            let self_waker = unsafe { &mut *self.waker.get() };

            // From embassy
            // https://github.com/embassy-rs/embassy/blob/b99533607ceed225dd12ae73aaa9a0d969a7365e/embassy-sync/src/waitqueue/waker.rs#L59-L61
            match self_waker {
                // Optimization: If both the old and new Wakers wake the same task, we can simply
                // keep the old waker, skipping the clone. (In most executor implementations,
                // cloning a waker is somewhat expensive, comparable to cloning an Arc).
                Some(ref w2) if (w2.will_wake(new_waker)) => {}
                _ => {
                    // clone the new waker and store it
                    if let Some(old_waker) = core::mem::replace(self_waker, Some(new_waker.clone()))
                    {
                        // We had a waker registered for another task. Wake it, so the other task can
                        // reregister itself if it's still interested.
                        //
                        // If two tasks are waiting on the same thing concurrently, this will cause them
                        // to wake each other in a loop fighting over this WakerRegistration. This wastes
                        // CPU but things will still work.
                        //
                        // If the user wants to have two tasks waiting on the same thing they should use
                        // a more appropriate primitive that can store multiple wakers.
                        old_waker.wake()
                    }
                }
            }
        });
    }

    /// Wake the waker.
    pub fn wake(&self) {
        critical_section::with(|_| {
            // SAFETY: This access is protected by the critical section.
            let self_waker = unsafe { &mut *self.waker.get() };
            if let Some(waker) = self_waker.take() {
                waker.wake()
            }
        });
    }
}
