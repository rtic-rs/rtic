//! [`Monotonic`](rtic_time::Monotonic) implementation for Silabs EFR32 and EFM32's RTCC ("Real
//! Time Clock with Capture") peripheral.
//!
//! Always runs at a fixed rate of 32768 Hz, which is a resolution of 30.518 µs.
//!
//! # Example
//!
//! ```
//! use rtic_monotonics::silabs::rtcc::prelude::*;
//!
//! // Create the type `Mono`. It will manage the RTCC peripheral,
//! // which is a 32768 Hz, 32-bit timer.
//! silabs_rtcc_monotonic!(Mono);
//!
//! fn init() {
//!     // Start the monotonic - passing the RTCC peripheral object, and
//!     // temporary access to the clock management unit.
//!     Mono::start(silabs_metapac::RTCC, &silabs_metapac::CMU);
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

/// Common definitions and traits for using the silabs rtc monotonic
pub mod prelude {
    pub use crate::silabs_rtcc_monotonic;
    pub use silabs_metapac;

    pub use crate::fugit::{self, ExtU64, ExtU64Ceil};
    pub use crate::Monotonic;
}

use core::sync::atomic::Ordering;

use crate::{rtic_time::timer_queue::TimerQueue, silabs::NVIC_PRIO_BITS, TimerQueueBackend};
use cortex_m::peripheral::NVIC;
use portable_atomic::AtomicU32;
use rtic_time::half_period_counter::calculate_now;

pub use silabs_metapac::{cmu_v1::Cmu, rtcc_v1::Rtcc};
use silabs_metapac::{
    rtcc_v1::vals::{Cc0CtrlMode, Cc1CtrlMode},
    Interrupt, RTCC,
};

/// Timer implementing [`TimerQueueBackend`].
pub struct TimerBackend;

impl TimerBackend {
    /// Starts the monotonic timer.
    ///
    /// **Do not use this function directly.**
    ///
    /// Use the prelude macros instead.
    pub fn _start(timer: Rtcc, cmu: &Cmu) {
        // enable required bus clock
        cmu.clken0().modify(|w| w.set_rtcc(true));

        // enable rtcc
        timer.en().write(|w| w.set_en(true));
        timer.cmd().write(|w| w.set_start(true));

        // enable interrupts
        // CC0: Dynamic wakeup, CC1: Half-period interrupt, OF: Overflow
        timer.ien().modify(|w| {
            w.set_cc0(true);
            w.set_cc1(true);
            w.set_of(true)
        });

        // configure half-period compare register to be triggered exactly
        // when have of the period's time has passed
        timer
            .cc1_ctrl()
            .write(|w| w.set_mode(Cc1CtrlMode::Outputcompare));
        timer.cc1_ocvalue().write(|w| w.set_oc(0x8000_0000));

        TIMER_QUEUE.initialize(Self {});

        unsafe {
            crate::set_monotonic_prio(NVIC_PRIO_BITS, Interrupt::RTCC);
            NVIC::unmask(Interrupt::RTCC);
        }
    }
}

static TIMER_QUEUE: TimerQueue<TimerBackend> = TimerQueue::new();

// used to widen rtcc timer from 32 to 64 bits
// maximum value is (64 - 32) bit * 2 (due to half period) = 33bit
static RTCC_HALF_PERIOD_COUNTER: AtomicU32 = AtomicU32::new(0);

impl TimerQueueBackend for TimerBackend {
    type Ticks = u64;

    fn now() -> Self::Ticks {
        let timer = RTCC;

        calculate_now(
            || RTCC_HALF_PERIOD_COUNTER.load(Ordering::Relaxed),
            || timer.cnt().read().cnt(),
        )
    }

    fn set_compare(instant: Self::Ticks) {
        const RTCC_MAX: u64 = 0xFFFF_FFFF;

        RTCC.cc0_ctrl()
            .write(|w| w.set_mode(Cc0CtrlMode::Outputcompare));

        let now = Self::now();
        let diff = instant.wrapping_sub(now);
        let compare_value = if diff <= RTCC_MAX {
            // the compare event will be triggered within the next RTCC period
            // so we can schedule it
            (instant & RTCC_MAX) as u32
        } else {
            // else, schedule wakeup for next overflow to re-evaluate if `instant` is now within one period
            0
        };

        RTCC.cc0_ocvalue().write(|w| w.set_oc(compare_value));
    }

    fn clear_compare_flag() {
        // clear interrupt flag
        RTCC.if_clr().write(|w| w.set_cc0(true));

        // disable compare
        RTCC.cc0_ctrl().write(|w| w.set_mode(Cc0CtrlMode::Off));
    }

    fn on_interrupt() {
        let interrupt_flag = RTCC.if_().read();

        // half period interrupt
        if interrupt_flag.cc1() {
            RTCC.if_clr().write(|w| w.set_cc1(true));

            let prev = RTCC_HALF_PERIOD_COUNTER.fetch_add(1, Ordering::Relaxed);
            ::core::assert!(
                prev.is_multiple_of(2),
                "Monotonic must have skipped an interrupt!"
            );
        }

        // overflow interrupt
        if interrupt_flag.of() {
            RTCC.if_clr().write(|w| w.set_of(true));

            let prev = RTCC_HALF_PERIOD_COUNTER.fetch_add(1, Ordering::Relaxed);
            ::core::assert!(
                !prev.is_multiple_of(2),
                "Monotonic must have skipped an interrupt!"
            );
        }
    }

    fn pend_interrupt() {
        NVIC::pend(Interrupt::RTCC);
    }

    fn timer_queue() -> &'static TimerQueue<Self> {
        &TIMER_QUEUE
    }
}

/// Create a EFR32/EFM32 RTCC based monotonic and register the necessary interrupt for it.
///
/// See [`crate::silabs::rtcc`] for more details.
///
/// # Arguments
///
/// * `name` - The name that the monotonic type will have.
#[macro_export]
macro_rules! silabs_rtcc_monotonic {
    ($name:ident) => {
        use $crate::{fugit, rtic_time};

        /// A `Monotonic` based on the Silabs's RTCC peripheral.
        pub struct $name;

        impl $name {
            /// Starts the `Monotonic`.
            ///
            /// This method must be called only once.
            pub fn start(timer: $crate::silabs::rtcc::Rtcc, cmu: &$crate::silabs::rtcc::Cmu) {
                #[no_mangle]
                #[allow(non_snake_case)]
                unsafe extern "C" fn RTCC() {
                    use $crate::TimerQueueBackend;
                    $crate::silabs::rtcc::TimerBackend::timer_queue().on_monotonic_interrupt();
                }

                $crate::silabs::rtcc::TimerBackend::_start(timer, cmu);
            }
        }

        impl $crate::TimerQueueBasedMonotonic for $name {
            type Backend = $crate::silabs::rtcc::TimerBackend;
            type Instant =
                fugit::Instant<<Self::Backend as $crate::TimerQueueBackend>::Ticks, 1, 32768>;
            type Duration =
                fugit::Duration<<Self::Backend as $crate::TimerQueueBackend>::Ticks, 1, 32768>;
        }

        rtic_time::impl_embedded_hal_delay_fugit!($name);
        rtic_time::impl_embedded_hal_async_delay_fugit!($name);
    };
}
