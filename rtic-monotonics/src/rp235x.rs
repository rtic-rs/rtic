//! [`Monotonic`](rtic_time::Monotonic) implementation for RP235x's Timer peripheral
//!
//!
//! Always runs at a fixed rate of 1 MHz.
//!
//! # Example
//!
//! ```
//! use rtic_monotonics::rp235x::prelude::*;
//!
//! rp235x_timer_monotonic!(Mono);
//!
//! fn init() {
//!     # // This is normally provided by the selected PAC
//!     # let timer = unsafe { core::mem::transmute(()) };
//!     # let mut resets = unsafe { core::mem::transmute(()) };
//!     #
//!     // Start the monotonic
//!     Mono::start(timer, &mut resets);
//! }
//!
//! async fn usage() {
//!     loop {
//!          // Use the monotonic
//!          let timestamp = Mono::now();
//!          Mono::delay(100.millis()).await;
//!     }
//! }
//! ```

/// Common definitions and traits for using the RP235x timer monotonic
pub mod prelude {
    pub use crate::rp235x_timer_monotonic;

    pub use crate::Monotonic;

    pub use fugit::{self, ExtU64, ExtU64Ceil};
}

use crate::TimerQueueBackend;
use cortex_m::peripheral::NVIC;
use rp235x_pac::Interrupt;
pub use rp235x_pac::{timer0, RESETS, TIMER0};
use rtic_time::timer_queue::TimerQueue;

/// Timer implementing [`TimerQueueBackend`].
pub struct TimerBackend;

impl TimerBackend {
    /// Starts the monotonic timer.
    ///
    /// **Do not use this function directly.**
    ///
    /// Use the prelude macros instead.
    pub fn _start(timer: TIMER0, resets: &RESETS) {
        resets.reset().modify(|_, w| w.timer0().clear_bit());
        while resets.reset_done().read().timer0().bit_is_clear() {}
        timer.inte().modify(|_, w| w.alarm_0().bit(true));

        TIMER_QUEUE.initialize(Self {});

        unsafe {
            crate::set_monotonic_prio(rp235x_pac::NVIC_PRIO_BITS, Interrupt::TIMER0_IRQ_0);
            NVIC::unmask(Interrupt::TIMER0_IRQ_0);
        }
    }

    fn timer() -> &'static timer0::RegisterBlock {
        unsafe { &*TIMER0::ptr() }
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
                break ((u64::from(hi0) << 32) | u64::from(low));
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
        NVIC::pend(Interrupt::TIMER0_IRQ_0);
    }

    fn timer_queue() -> &'static TimerQueue<Self> {
        &TIMER_QUEUE
    }
}

/// Create an RP235x timer based monotonic and register the necessary interrupt for it.
///
/// See [`crate::rp235x`] for more details.
///
/// # Arguments
///
/// * `name` - The name that the monotonic type will have.
#[macro_export]
macro_rules! rp235x_timer_monotonic {
    ($name:ident) => {
        /// A `Monotonic` based on the RP235x Timer peripheral.
        pub struct $name;

        impl $name {
            /// Starts the `Monotonic`.
            ///
            /// This method must be called only once.
            pub fn start(timer: $crate::rp235x::TIMER0, resets: &$crate::rp235x::RESETS) {
                #[no_mangle]
                #[allow(non_snake_case)]
                unsafe extern "C" fn TIMER0_IRQ_0() {
                    use $crate::TimerQueueBackend;
                    $crate::rp235x::TimerBackend::timer_queue().on_monotonic_interrupt();
                }

                $crate::rp235x::TimerBackend::_start(timer, resets);
            }
        }

        impl $crate::TimerQueueBasedMonotonic for $name {
            type Backend = $crate::rp235x::TimerBackend;
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
