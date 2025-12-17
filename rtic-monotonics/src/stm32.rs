//! [`Monotonic`](rtic_time::Monotonic) implementations for STM32 chips.
//!
//! Not all timers are available on all parts. Ensure that only available
//! timers are exposed by having the correct `stm32*` feature enabled for `rtic-monotonics`.
//!
//! # Example
//!
//! ```
//! use rtic_monotonics::stm32::prelude::*;
//!
//! // Create the type `Mono`. It will manage the TIM2 timer, and
//! // run with a resolution of 1 µs (1,000,000 ticks per second).
//! stm32_tim2_monotonic!(Mono, 1_000_000);
//!
//! fn init() {
//!     // If using `embassy-stm32` HAL, timer clock can be read out like this:
//!     let timer_clock_hz = embassy_stm32::peripherals::TIM2::frequency();
//!     // Or define it manually if you are using another HAL or know the
//!     // correct frequency:
//!     let timer_clock_hz = 64_000_000;
//!
//!     // Start the monotonic. The TIM2 prescaler is calculated from the
//!     // clock frequency given here, and the resolution given to the
//!     // `stm32_tim2_monotonic!` macro call above. No PAC object is required.
//!     Mono::start(timer_clock_hz);
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

/// Common definitions and traits for using the STM32 monotonics
pub mod prelude {
    #[cfg(feature = "stm32_tim2")]
    pub use crate::stm32_tim2_monotonic;

    #[cfg(feature = "stm32_tim3")]
    pub use crate::stm32_tim3_monotonic;

    #[cfg(feature = "stm32_tim4")]
    pub use crate::stm32_tim4_monotonic;

    #[cfg(feature = "stm32_tim5")]
    pub use crate::stm32_tim5_monotonic;

    #[cfg(feature = "stm32_tim15")]
    pub use crate::stm32_tim15_monotonic;

    pub use crate::Monotonic;
    pub use fugit::{self, ExtU64, ExtU64Ceil};
}

use crate::atomic::{AtomicU64, Ordering};
use rtic_time::{
    half_period_counter::calculate_now,
    timer_queue::{TimerQueue, TimerQueueBackend},
};
use stm32_metapac as pac;

mod _generated {
    #![allow(dead_code)]
    #![allow(unused_imports)]
    #![allow(non_snake_case)]

    include!(concat!(env!("OUT_DIR"), "/_generated.rs"));
}

#[doc(hidden)]
#[macro_export]
macro_rules! __internal_create_stm32_timer_interrupt {
    ($mono_backend:ident, $interrupt_name:ident) => {
        #[no_mangle]
        #[allow(non_snake_case)]
        unsafe extern "C" fn $interrupt_name() {
            use $crate::TimerQueueBackend;
            $crate::stm32::$mono_backend::timer_queue().on_monotonic_interrupt();
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __internal_create_stm32_timer_struct {
    ($name:ident, $mono_backend:ident, $timer:ident, $tick_rate_hz:expr) => {
        /// A `Monotonic` based on an STM32 timer peripheral.
        pub struct $name;

        impl $name {
            /// Starts the `Monotonic`.
            ///
            /// - `tim_clock_hz`: `TIMx` peripheral clock frequency.
            ///
            /// Panics if it is impossible to achieve the desired monotonic tick rate based
            /// on the given `tim_clock_hz` parameter. If that happens, adjust the desired monotonic tick rate.
            ///
            /// This method must be called only once.
            pub fn start(tim_clock_hz: u32) {
                $crate::__internal_create_stm32_timer_interrupt!($mono_backend, $timer);

                $crate::stm32::$mono_backend::_start(tim_clock_hz, $tick_rate_hz);
            }
        }

        impl $crate::TimerQueueBasedMonotonic for $name {
            type Backend = $crate::stm32::$mono_backend;
            type Instant = $crate::fugit::Instant<
                <Self::Backend as $crate::TimerQueueBackend>::Ticks,
                1,
                { $tick_rate_hz },
            >;
            type Duration = $crate::fugit::Duration<
                <Self::Backend as $crate::TimerQueueBackend>::Ticks,
                1,
                { $tick_rate_hz },
            >;
        }

        $crate::rtic_time::impl_embedded_hal_delay_fugit!($name);
        $crate::rtic_time::impl_embedded_hal_async_delay_fugit!($name);
    };
}

/// Create a TIM2 based monotonic and register the TIM2 interrupt for it.
///
/// See [`crate::stm32`] for more details.
///
/// # Arguments
///
/// * `name` - The name that the monotonic type will have.
/// * `tick_rate_hz` - The tick rate of the timer peripheral.
///
#[cfg(feature = "stm32_tim2")]
#[macro_export]
macro_rules! stm32_tim2_monotonic {
    ($name:ident, $tick_rate_hz:expr) => {
        $crate::__internal_create_stm32_timer_struct!($name, Tim2Backend, TIM2, $tick_rate_hz);
    };
}

/// Create a TIM3 based monotonic and register the TIM3 interrupt for it.
///
/// See [`crate::stm32`] for more details.
///
/// # Arguments
///
/// * `name` - The name that the monotonic type will have.
/// * `tick_rate_hz` - The tick rate of the timer peripheral.
///
#[cfg(feature = "stm32_tim3")]
#[macro_export]
macro_rules! stm32_tim3_monotonic {
    ($name:ident, $tick_rate_hz:expr) => {
        $crate::__internal_create_stm32_timer_struct!($name, Tim3Backend, TIM3, $tick_rate_hz);
    };
}

/// Create a TIM4 based monotonic and register the TIM4 interrupt for it.
///
/// See [`crate::stm32`] for more details.
///
/// # Arguments
///
/// * `name` - The name that the monotonic type will have.
/// * `tick_rate_hz` - The tick rate of the timer peripheral.
///
#[cfg(feature = "stm32_tim4")]
#[macro_export]
macro_rules! stm32_tim4_monotonic {
    ($name:ident, $tick_rate_hz:expr) => {
        $crate::__internal_create_stm32_timer_struct!($name, Tim4Backend, TIM4, $tick_rate_hz);
    };
}

/// Create a TIM5 based monotonic and register the TIM5 interrupt for it.
///
/// See [`crate::stm32`] for more details.
///
/// # Arguments
///
/// * `name` - The name that the monotonic type will have.
/// * `tick_rate_hz` - The tick rate of the timer peripheral.
///
#[cfg(feature = "stm32_tim5")]
#[macro_export]
macro_rules! stm32_tim5_monotonic {
    ($name:ident, $tick_rate_hz:expr) => {
        $crate::__internal_create_stm32_timer_struct!($name, Tim5Backend, TIM5, $tick_rate_hz);
    };
}

/// Create a TIM15 based monotonic and register the TIM15 interrupt for it.
///
/// See [`crate::stm32`] for more details.
///
/// # Arguments
///
/// * `name` - The name that the monotonic type will have.
/// * `tick_rate_hz` - The tick rate of the timer peripheral.
///
#[cfg(feature = "stm32_tim15")]
#[macro_export]
macro_rules! stm32_tim15_monotonic {
    ($name:ident, $tick_rate_hz:expr) => {
        $crate::__internal_create_stm32_timer_struct!($name, Tim15Backend, TIM15, $tick_rate_hz);
    };
}

macro_rules! make_timer {
    ($backend_name:ident, $timer:ident, $bits:ident, $overflow:ident, $tq:ident$(, doc: ($($doc:tt)*))?) => {
        /// Monotonic timer backend implementation.
        $(
            #[cfg_attr(docsrs, doc(cfg($($doc)*)))]
        )?

        pub struct $backend_name;

        use pac::$timer;

        static $overflow: AtomicU64 = AtomicU64::new(0);
        static $tq: TimerQueue<$backend_name> = TimerQueue::new();

        impl $backend_name {
            /// Starts the timer.
            ///
            /// **Do not use this function directly.**
            ///
            /// Use the prelude macros instead.
            pub fn _start(tim_clock_hz: u32, timer_hz: u32) {
                _generated::$timer::enable();
                _generated::$timer::reset();

                $timer.cr1().modify(|r| r.set_cen(false));

                assert!((tim_clock_hz % timer_hz) == 0, "Unable to find suitable timer prescaler value!");
                let Ok(psc) = u16::try_from(tim_clock_hz / timer_hz - 1) else {
                    panic!("Clock prescaler overflowed!");
                };
                $timer.psc().write(|r| r.set_psc(psc));

                // Enable full-period interrupt.
                $timer.dier().modify(|r| r.set_uie(true));

                // Configure and enable half-period interrupt
                $timer.ccr(0).write(|r| r.set_ccr(($bits::MAX - ($bits::MAX >> 1)).into()));
                $timer.dier().modify(|r| r.set_ccie(0, true));

                // Trigger an update event to load the prescaler value to the clock.
                $timer.egr().write(|r| r.set_ug(true));

                // Clear timer value so it is known that we are at the first half period
                $timer.cnt().write(|r| r.set_cnt(1));

                // Triggering the update event might have raised overflow interrupts.
                // Clear them to return to a known state.
                $timer.sr().write(|r| {
                    r.0 = !0;
                    r.set_uif(false);
                    r.set_ccif(0, false);
                    r.set_ccif(1, false);
                });

                $tq.initialize(Self {});
                $overflow.store(0, Ordering::SeqCst);

                // Start the counter.
                $timer.cr1().modify(|r| {
                    r.set_cen(true);
                });

                // SAFETY: We take full ownership of the peripheral and interrupt vector,
                // plus we are not using any external shared resources so we won't impact
                // basepri/source masking based critical sections.
                unsafe {
                    crate::set_monotonic_prio(_generated::NVIC_PRIO_BITS, pac::Interrupt::$timer);
                    cortex_m::peripheral::NVIC::unmask(pac::Interrupt::$timer);
                }
            }
        }

        impl TimerQueueBackend for $backend_name {
            type Ticks = u64;

            fn now() -> Self::Ticks {
                calculate_now(
                    || $overflow.load(Ordering::Relaxed),
                    || $timer.cnt().read().cnt()
                )
            }

            fn set_compare(instant: Self::Ticks) {
                let now = Self::now();

                // Since the timer may or may not overflow based on the requested compare val, we check how many ticks are left.
                // `wrapping_sub` takes care of the u64 integer overflow special case.
                let val = if instant.wrapping_sub(now) <= ($bits::MAX as u64) {
                    instant as $bits
                } else {
                    // In the past or will overflow
                    0
                };

                $timer.ccr(1).write(|r| r.set_ccr(val.into()));
            }

            fn clear_compare_flag() {
                $timer.sr().write(|r| {
                    r.0 = !0;
                    r.set_ccif(1, false);
                });
            }

            fn pend_interrupt() {
                cortex_m::peripheral::NVIC::pend(pac::Interrupt::$timer);
            }

            fn enable_timer() {
                $timer.dier().modify(|r| r.set_ccie(1, true));
            }

            fn disable_timer() {
                $timer.dier().modify(|r| r.set_ccie(1, false));
            }

            fn on_interrupt() {
                // Full period
                if $timer.sr().read().uif() {
                    $timer.sr().write(|r| {
                        r.0 = !0;
                        r.set_uif(false);
                    });
                    let prev = $overflow.fetch_add(1, Ordering::Relaxed);
                    assert!(prev % 2 == 1, "Monotonic must have missed an interrupt!");
                }
                // Half period
                if $timer.sr().read().ccif(0) {
                    $timer.sr().write(|r| {
                        r.0 = !0;
                        r.set_ccif(0, false);
                    });
                    let prev = $overflow.fetch_add(1, Ordering::Relaxed);
                    assert!(prev % 2 == 0, "Monotonic must have missed an interrupt!");
                }
            }

            fn timer_queue() -> &'static TimerQueue<$backend_name> {
                &$tq
            }
        }
    };
}

#[cfg(feature = "stm32_tim2")]
make_timer!(Tim2Backend, TIM2, u32, TIMER2_OVERFLOWS, TIMER2_TQ);

#[cfg(feature = "stm32_tim3")]
make_timer!(Tim3Backend, TIM3, u16, TIMER3_OVERFLOWS, TIMER3_TQ);

#[cfg(feature = "stm32_tim4")]
make_timer!(Tim4Backend, TIM4, u16, TIMER4_OVERFLOWS, TIMER4_TQ);

#[cfg(feature = "stm32_tim5")]
make_timer!(Tim5Backend, TIM5, u16, TIMER5_OVERFLOWS, TIMER5_TQ);

#[cfg(feature = "stm32_tim15")]
make_timer!(Tim15Backend, TIM15, u16, TIMER15_OVERFLOWS, TIMER15_TQ);
