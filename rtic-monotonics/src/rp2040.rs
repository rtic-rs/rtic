//! [`Monotonic`](rtic_time::Monotonic) implementation for RP2040's Timer peripheral.
//!
//! Always runs at a fixed rate of 1 MHz.
//!
//! # Example
//!
//! ```
//! use rtic_monotonics::rp2040::prelude::*;
//!
//! // Create the type `Mono`. It will manage the TIMER peripheral,
//! // which is a 1 MHz, 64-bit timer.
//! rp2040_timer_monotonic!(Mono);
//!
//! fn init() {
//!     # // This is normally provided by the selected PAC
//!     # let TIMER = unsafe { core::mem::transmute(()) };
//!     # let mut RESETS = unsafe { core::mem::transmute(()) };
//!     #
//!     // Start the monotonic - passing ownership of an rp2040_pac object for
//!     // TIMER0, and temporary access to one for the RESET peripheral.
//!     Mono::start(TIMER, &mut RESETS);
//! }
//!
//! async fn usage() {
//!     loop {
//!          // You can use the monotonic to get the time...
//!          let timestamp = Mono::now();
//!          // ...and you can use it to add a delay to this async function
//!          Mono::delay(100.millis()).await;
//!     }
//! }
//! ```

/// Common definitions and traits for using the RP2040 timer monotonic
pub mod prelude {
    pub use crate::rp2040_timer_monotonic;

    pub use crate::Monotonic;

    pub use fugit::{self, ExtU64, ExtU64Ceil};
}

use crate::TimerQueueBackend;
use rp2040_pac::{timer, Interrupt, NVIC};
pub use rp2040_pac::{RESETS, TIMER};
use rtic_time::timer_queue::TimerQueue;

/// Timer implementing [`TimerQueueBackend`].
pub struct TimerBackend;

impl TimerBackend {
    /// Starts the monotonic timer.
    ///
    /// **Do not use this function directly.**
    ///
    /// Use the prelude macros instead.
    pub fn _start(timer: TIMER, resets: &RESETS) {
        resets.reset().modify(|_, w| w.timer().clear_bit());
        while resets.reset_done().read().timer().bit_is_clear() {}
        timer.inte().modify(|_, w| w.alarm_0().bit(true));

        TIMER_QUEUE.initialize(Self {});

        unsafe {
            crate::set_monotonic_prio(rp2040_pac::NVIC_PRIO_BITS, Interrupt::TIMER_IRQ_0);
            NVIC::unmask(Interrupt::TIMER_IRQ_0);
        }
    }

    fn timer() -> &'static timer::RegisterBlock {
        unsafe { &*TIMER::ptr() }
    }
}

static TIMER_QUEUE: TimerQueue<TimerBackend> = TimerQueue::new();

impl TimerQueueBackend for TimerBackend {
    type Ticks = u64;

    fn now() -> Self::Ticks {
        let timer = Self::timer();

        let mut hi0 = timer.timerawh().read().bits();
        loop {
            let low = timer.timerawl().read().bits();
            let hi1 = timer.timerawh().read().bits();
            if hi0 == hi1 {
                break (u64::from(hi0) << 32) | u64::from(low);
            }
            hi0 = hi1;
        }
    }

    fn set_compare(instant: Self::Ticks) {
        let now = Self::now();

        const MAX: u64 = u32::MAX as u64;

        // Since the timer may or may not overflow based on the requested compare val, we check
        // how many ticks are left.
        // `wrapping_sub` takes care of the u64 integer overflow special case.
        let val = if instant.wrapping_sub(now) <= MAX {
            instant & MAX
        } else {
            0
        };

        Self::timer()
            .alarm0()
            .write(|w| unsafe { w.bits(val as u32) });
    }

    fn clear_compare_flag() {
        Self::timer().intr().modify(|_, w| w.alarm_0().bit(true));
    }

    fn pend_interrupt() {
        rp2040_pac::NVIC::pend(Interrupt::TIMER_IRQ_0);
    }

    fn timer_queue() -> &'static TimerQueue<Self> {
        &TIMER_QUEUE
    }
}

/// Create an RP2040 timer based monotonic and register the necessary interrupt for it.
///
/// See [`crate::rp2040`] for more details.
///
/// # Arguments
///
/// * `name` - The name that the monotonic type will have.
#[macro_export]
macro_rules! rp2040_timer_monotonic {
    ($name:ident) => {
        /// A `Monotonic` based on the RP2040 Timer peripheral.
        pub struct $name;

        impl $name {
            /// Starts the `Monotonic`.
            ///
            /// This method must be called only once.
            pub fn start(timer: $crate::rp2040::TIMER, resets: &$crate::rp2040::RESETS) {
                #[no_mangle]
                #[allow(non_snake_case)]
                unsafe extern "C" fn TIMER_IRQ_0() {
                    use $crate::TimerQueueBackend;
                    $crate::rp2040::TimerBackend::timer_queue().on_monotonic_interrupt();
                }

                $crate::rp2040::TimerBackend::_start(timer, resets);
            }
        }

        impl $crate::TimerQueueBasedMonotonic for $name {
            type Backend = $crate::rp2040::TimerBackend;
            type Instant = $crate::fugit::Instant<
                <Self::Backend as $crate::TimerQueueBackend>::Ticks,
                1,
                1_000_000,
            >;
            type Duration = $crate::fugit::Duration<
                <Self::Backend as $crate::TimerQueueBackend>::Ticks,
                1,
                1_000_000,
            >;
        }

        $crate::rtic_time::impl_embedded_hal_delay_fugit!($name);
        $crate::rtic_time::impl_embedded_hal_async_delay_fugit!($name);
    };
}
