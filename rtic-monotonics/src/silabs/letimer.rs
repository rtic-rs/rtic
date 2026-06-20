//! [`Monotonic`](rtic_time::Monotonic) implementation for Silabs EFR32 and EFM32's 24 bit LETimer ("Low Energy Timer") peripheral.
//!
//! Always runs at a fixed rate of 32768 Hz, which is a resolution of 30.518 µs.
//!
//! # Example
//!
//! ```
//! use rtic_monotonics::silabs::prelude::*;
//!
//! // Create the type `Mono`. It will manage the LETimer peripheral,
//! // which is a 24-bit timer, powered by a 32768Hz oscillator (by default).
//! // You can optionally specify a different frequency (e.g. 8092) by passing
//! // it to the `silabs_letimer_monotonic!` macro.
//! silabs_letimer_monotonic!(Mono, 8092);
//!
//! fn init() {
//!     # // This is normally provided by the selected PAC
//!     # let peripherals = unsafe { core::mem::transmute(()) };
//!     #
//!     // Start the monotonic - passing ownership of a silabs_metapac object for
//!     // LETimer, and temporary access of the clock management unit.
//!     Mono::start(peripherals.letimer0_ns, peripherals.cmu_ns);
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

/// Common definitions and traits for using the silabs letimer monotonic
pub mod prelude {
    pub use crate::silabs_letimer_monotonic;
    pub use silabs_metapac;

    pub use crate::fugit::{self, ExtU64, ExtU64Ceil};
    pub use crate::Monotonic;
}

use core::sync::atomic::Ordering;

use crate::{
    rtic_time::timer_queue::TimerQueue, set_monotonic_prio, silabs::NVIC_PRIO_BITS,
    TimerQueueBackend,
};
use cortex_m::peripheral::NVIC;
use portable_atomic::AtomicU32;
use rtic_time::half_period_counter::calculate_now;
use silabs_metapac::{Interrupt, LETIMER0};

#[cfg(feature = "silabs-efr32mg22")]
pub use silabs_metapac::{
    cmu_v1::Cmu,
    letimer_v0::{vals::Cntpresc, Letimer},
};
#[cfg(feature = "silabs-efr32mg24")]
pub use silabs_metapac::{
    cmu_v3::Cmu,
    letimer_v1::{vals::Cntpresc, Letimer},
};
#[cfg(feature = "silabs-efr32fg25")]
pub use silabs_metapac::{
    cmu_v4::Cmu,
    letimer_v1::{vals::Cntpresc, Letimer},
};
#[cfg(feature = "silabs-efr32mg26")]
pub use silabs_metapac::{
    cmu_v7::Cmu,
    letimer_v1::{vals::Cntpresc, Letimer},
};

/// Timer implementing [`TimerQueueBackend`].
pub struct TimerBackend;

const U24_MAX: u32 = 0xFF_FFFF;
const HALF_PERIOD_UP: u32 = 0x80_0000;
// as the letimer counts downwards, the middle of the period (where the
// half-period interrupt should be triggered) is at 0x7F_FFFF, and not 0x80_0000
const HALF_PERIOD_DOWN: u32 = U24_MAX - HALF_PERIOD_UP;

impl TimerBackend {
    /// Starts the monotonic timer.
    ///
    /// **Do not use this function directly.**
    ///
    /// Use the prelude macros instead.
    pub fn _start(timer: Letimer, cmu: &Cmu, tick_rate_hz: u32) {
        // enable required bus clock
        cmu.clken0().modify(|w| w.set_letimer0(true));

        // configure prescaling
        timer.ctrl().write(|w| match tick_rate_hz {
            32_768 => w.set_cntpresc(Cntpresc::Div1),
            16_384 => w.set_cntpresc(Cntpresc::Div2),
            8_192 => w.set_cntpresc(Cntpresc::Div4),
            4_096 => w.set_cntpresc(Cntpresc::Div8),
            2_048 => w.set_cntpresc(Cntpresc::Div16),
            1_024 => w.set_cntpresc(Cntpresc::Div32),
            512 => w.set_cntpresc(Cntpresc::Div64),
            256 => w.set_cntpresc(Cntpresc::Div128),
            _ => ::core::panic!("Timer cannot run at desired tick rate!"),
        });

        // enable timer - should be done after ctrl config is done according to documentation
        timer.en().write(|w| w.set_en(true));

        // configure half-period compare register to be triggered exactly
        // when have of the period's time has passed
        timer.comp1().write(|w| w.set_comp1(HALF_PERIOD_DOWN));

        // enable interrupts
        // COMP0: Dynamic wakeup, COMP1: Half-period interrupt, UF: Underflow
        timer.ien().modify(|w| {
            w.set_comp0(true);
            w.set_comp1(true);
            w.set_uf(true)
        });

        timer.cmd().write(|w| w.set_start(true));

        TIMER_QUEUE.initialize(Self {});

        unsafe {
            set_monotonic_prio(NVIC_PRIO_BITS, Interrupt::LETIMER0);
            NVIC::unmask(Interrupt::LETIMER0);
        }
    }

    fn letimer() -> &'static Letimer {
        &LETIMER0
    }
}

static TIMER_QUEUE: TimerQueue<TimerBackend> = TimerQueue::new();

// used to widen letimer from 24 to 64 bits
// maximum value is (64 - 24) bit * 2 (due to half period) = 41bit
const LETIMER_HALF_PERIOD_COUNTER: AtomicU32 = AtomicU32::new(0);

impl TimerQueueBackend for TimerBackend {
    type Ticks = u64;

    fn now() -> Self::Ticks {
        let timer = Self::letimer();

        calculate_now(
            || LETIMER_HALF_PERIOD_COUNTER.load(Ordering::Relaxed),
            || {
                let now = timer.cnt().read().cnt();
                if now == 0 {
                    0 // timer not started yet
                } else {
                    // the LEtimer is counting downwards, from U24_MAX to 0.
                    // as instants are expected to go upwards (i.e. from 0 to U24_MAX)
                    // we invert the now value
                    U24_MAX - now
                }
            },
        )
    }

    fn set_compare(instant: Self::Ticks) {
        let now = Self::now();

        let compare_value = if instant.saturating_sub(now) <= U24_MAX.into() {
            // the compare event will be triggered within the next RTCC period
            // so we can schedule it
            U24_MAX - (instant as u32 & U24_MAX)
        } else {
            // else, schedule wakeup for next overflow to re-evaluate if `instant` is now within one period
            // as the LETimer counts downwards, the first value in each period is the max value
            U24_MAX
        };

        Self::letimer()
            .comp0()
            .write(|w| w.set_comp0(compare_value));
    }

    fn clear_compare_flag() {
        // clear interrupt flag
        Self::letimer().if_clr().write(|w| w.set_comp0(true));

        // disable compare
        Self::letimer().comp0().write(|w| w.set_comp0(0));
    }

    fn on_interrupt() {
        let interrupt_flag = Self::letimer().if_().read();

        // half period interrupt
        if interrupt_flag.comp1() {
            Self::letimer().if_clr().write(|w| w.set_comp1(true));

            let prev = LETIMER_HALF_PERIOD_COUNTER.fetch_add(1, Ordering::Relaxed);
            ::core::assert!(prev % 2 == 0, "Monotonic must have skipped an interrupt!");
        }

        // overflow interrupt
        if interrupt_flag.uf() {
            Self::letimer().if_clr().write(|w| w.set_uf(true));

            let prev = LETIMER_HALF_PERIOD_COUNTER.fetch_add(1, Ordering::Relaxed);
            ::core::assert!(prev % 2 == 1, "Monotonic must have skipped an interrupt!");
        }
    }

    fn pend_interrupt() {
        NVIC::pend(Interrupt::LETIMER0);
    }

    fn timer_queue() -> &'static TimerQueue<Self> {
        &TIMER_QUEUE
    }
}

/// Create a EFR32/EFM32 LETIMER based monotonic and register the necessary interrupt for it.
///
/// See [`crate::silabs::letimer`] for more details.
///
/// # Arguments
///
/// * `name` - The name that the monotonic type will have.
#[macro_export]
macro_rules! silabs_letimer_monotonic {
    ($name:ident) => {
        $crate::silabs_letimer_monotonic!($name, 32_768);
    };
    ($name:ident, $tick_rate_hz:expr) => {
        use $crate::{fugit, rtic_time};

        /// A `Monotonic` based on the Silabs's LETIMER peripheral.
        pub struct $name;

        impl $name {
            /// Starts the `Monotonic`.
            ///
            /// This method must be called only once.
            pub fn start(
                timer: $crate::silabs::letimer::Letimer,
                cmu: &$crate::silabs::letimer::Cmu,
            ) {
                #[no_mangle]
                #[allow(non_snake_case)]
                unsafe extern "C" fn LETIMER0() {
                    use $crate::TimerQueueBackend;
                    $crate::silabs::letimer::TimerBackend::timer_queue().on_monotonic_interrupt();
                }

                $crate::silabs::letimer::TimerBackend::_start(timer, cmu, $tick_rate_hz);
            }
        }

        impl $crate::TimerQueueBasedMonotonic for $name {
            type Backend = $crate::silabs::letimer::TimerBackend;
            type Instant = fugit::Instant<
                <Self::Backend as $crate::TimerQueueBackend>::Ticks,
                1,
                { $tick_rate_hz },
            >;
            type Duration = fugit::Duration<
                <Self::Backend as $crate::TimerQueueBackend>::Ticks,
                1,
                { $tick_rate_hz },
            >;
        }

        rtic_time::impl_embedded_hal_delay_fugit!($name);
        rtic_time::impl_embedded_hal_async_delay_fugit!($name);
    };
}
