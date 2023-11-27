//! [`Monotonic`] impl for the STM32.
//!
//! Not all timers are available on all parts. Ensure that only available
//! timers are exposed by having the correct `stm32*` feature enabled for `rtic-monotonics`.
//!
//! # Example
//!
//! ```
//! use rtic_monotonics::stm32::*;
//! use rtic_monotonics::stm32::Tim2 as Mono;
//! use rtic_monotonics::Monotonic;
//! use embassy_stm32::peripherals::TIM2;
//! use embassy_stm32::rcc::low_level::RccPeripheral;
//!
//! fn init() {
//!     // Generate timer token to ensure correct timer interrupt handler is used.
//!     let token = rtic_monotonics::create_stm32_tim2_monotonic_token!();
//!
//!     // If using `embassy-stm32` HAL, timer clock can be read out like this:
//!     let timer_clock_hz = TIM2::frequency();
//!     // Or define it manually if you are using other HAL or know correct frequency:
//!     let timer_clock_hz = 64_000_000;
//!
//!     // Start the monotonic
//!     Mono::start(timer_clock_hz, token);
//! }
//!
//! async fn usage() {
//!     loop {
//!          // Use the monotonic
//!          let timestamp = Mono::now().ticks();
//!          Mono::delay(100.millis()).await;
//!     }
//! }
//! ```

use crate::{Monotonic, TimeoutError, TimerQueue};
use atomic_polyfill::{compiler_fence, AtomicU64, Ordering};
pub use fugit::{self, ExtU64, ExtU64Ceil};
use stm32_metapac as pac;

mod _generated {
    #![allow(dead_code)]
    #![allow(unused_imports)]
    #![allow(non_snake_case)]

    include!(concat!(env!("OUT_DIR"), "/_generated.rs"));
}

const TIMER_HZ: u32 = 1_000_000;

#[doc(hidden)]
#[macro_export]
macro_rules! __internal_create_stm32_timer_interrupt {
    ($mono_timer:ident, $timer:ident, $timer_token:ident) => {{
        #[no_mangle]
        #[allow(non_snake_case)]
        unsafe extern "C" fn $timer() {
            $crate::stm32::$mono_timer::__tq().on_monotonic_interrupt();
        }

        pub struct $timer_token;

        unsafe impl $crate::InterruptToken<$crate::stm32::$mono_timer> for $timer_token {}

        $timer_token
    }};
}

/// Register TIM2 interrupt for the monotonic.
#[cfg(feature = "stm32_tim2")]
#[macro_export]
macro_rules! create_stm32_tim2_monotonic_token {
    () => {{
        $crate::__internal_create_stm32_timer_interrupt!(Tim2, TIM2, Tim2Token)
    }};
}

/// Register TIM3 interrupt for the monotonic.
#[cfg(feature = "stm32_tim3")]
#[macro_export]
macro_rules! create_stm32_tim3_monotonic_token {
    () => {{
        $crate::__internal_create_stm32_timer_interrupt!(Tim3, TIM3, Tim3Token)
    }};
}

/// Register TIM4 interrupt for the monotonic.
#[cfg(feature = "stm32_tim4")]
#[macro_export]
macro_rules! create_stm32_tim4_monotonic_token {
    () => {{
        $crate::__internal_create_stm32_timer_interrupt!(Tim4, TIM4, Tim4Token)
    }};
}

/// Register TIM5 interrupt for the monotonic.
#[cfg(feature = "stm32_tim5")]
#[macro_export]
macro_rules! create_stm32_tim5_monotonic_token {
    () => {{
        $crate::__internal_create_stm32_timer_interrupt!(Tim5, TIM5, Tim5Token)
    }};
}

/// Register TIM12 interrupt for the monotonic.
#[cfg(feature = "stm32_tim12")]
#[macro_export]
macro_rules! create_stm32_tim12_monotonic_token {
    () => {{
        $crate::__internal_create_stm32_timer_interrupt!(Tim12, TIM12, Tim12Token)
    }};
}

/// Register TIM15 interrupt for the monotonic.
#[cfg(feature = "stm32_tim15")]
#[macro_export]
macro_rules! create_stm32_tim15_monotonic_token {
    () => {{
        $crate::__internal_create_stm32_timer_interrupt!(Tim15, TIM15, Tim15Token)
    }};
}

macro_rules! make_timer {
    ($mono_name:ident, $timer:ident, $bits:ident, $overflow:ident, $tq:ident$(, doc: ($($doc:tt)*))?) => {
        /// Monotonic timer queue implementation.
        $(
            #[cfg_attr(docsrs, doc(cfg($($doc)*)))]
        )?

        pub struct $mono_name;

        use pac::$timer;

        static $overflow: AtomicU64 = AtomicU64::new(0);
        static $tq: TimerQueue<$mono_name> = TimerQueue::new();

        impl $mono_name {
            /// Starts the monotonic timer.
            ///
            /// - `tim_clock_hz`: `TIMx` peripheral clock frequency.
            /// - `_interrupt_token`: Required for correct timer interrupt handling.
            ///
            /// This method must be called only once.
            pub fn start(tim_clock_hz: u32, _interrupt_token: impl crate::InterruptToken<Self>) {
                _generated::$timer::enable();
                _generated::$timer::reset();

                $timer.cr1().modify(|r| r.set_cen(false));

                assert!((tim_clock_hz % TIMER_HZ) == 0, "Unable to find suitable timer prescaler value!");
                let psc = tim_clock_hz / TIMER_HZ - 1;
                $timer.psc().write(|r| r.set_psc(psc as u16));

                // Enable full-period interrupt.
                $timer.dier().modify(|r| r.set_uie(true));

                // Configure and enable half-period interrupt
                $timer.ccr(2).write(|r| r.set_ccr($bits::MAX - ($bits::MAX >> 1)));
                $timer.dier().modify(|r| r.set_ccie(2, true));

                // Trigger an update event to load the prescaler value to the clock.
                $timer.egr().write(|r| r.set_ug(true));

                // The above line raises an update event which will indicate that the timer is already finished.
                // Since this is not the case, it should be cleared.
                $timer.sr().modify(|r| r.set_uif(false));

                // Start the counter.
                $timer.cr1().modify(|r| {
                    r.set_cen(true);
                });

                $tq.initialize(Self {});

                // SAFETY: We take full ownership of the peripheral and interrupt vector,
                // plus we are not using any external shared resources so we won't impact
                // basepri/source masking based critical sections.
                unsafe {
                    crate::set_monotonic_prio(_generated::NVIC_PRIO_BITS, pac::Interrupt::$timer);
                    cortex_m::peripheral::NVIC::unmask(pac::Interrupt::$timer);
                }
            }

            /// Used to access the underlying timer queue
            #[doc(hidden)]
            pub fn __tq() -> &'static TimerQueue<$mono_name> {
                &$tq
            }

            /// Delay for some duration of time.
            #[inline]
            pub async fn delay(duration: <Self as Monotonic>::Duration) {
                $tq.delay(duration).await;
            }

            /// Timeout at a specific time.
            pub async fn timeout_at<F: core::future::Future>(
                instant: <Self as rtic_time::Monotonic>::Instant,
                future: F,
            ) -> Result<F::Output, TimeoutError> {
                $tq.timeout_at(instant, future).await
            }

            /// Timeout after a specific duration.
            #[inline]
            pub async fn timeout_after<F: core::future::Future>(
                duration: <Self as Monotonic>::Duration,
                future: F,
            ) -> Result<F::Output, TimeoutError> {
                $tq.timeout_after(duration, future).await
            }

            /// Delay to some specific time instant.
            #[inline]
            pub async fn delay_until(instant: <Self as Monotonic>::Instant) {
                $tq.delay_until(instant).await;
            }
        }

        #[cfg(feature = "embedded-hal-async")]
        impl embedded_hal_async::delay::DelayUs for $mono_name {
            #[inline]
            async fn delay_us(&mut self, us: u32) {
                Self::delay(u64::from(us).micros_at_least()).await;
            }

            #[inline]
            async fn delay_ms(&mut self, ms: u32) {
                Self::delay(u64::from(ms).millis_at_least()).await;
            }
        }

        impl embedded_hal::delay::DelayUs for $mono_name {
            fn delay_us(&mut self, us: u32) {
                let done = Self::now() + u64::from(us).micros_at_least() + Self::TICK_PERIOD;
                while Self::now() < done {}
            }
            fn delay_ms(&mut self, ms: u32) {
                let done = Self::now() + u64::from(ms).millis_at_least() + Self::TICK_PERIOD;
                while Self::now() < done {}
            }
        }

        impl Monotonic for $mono_name {
            type Instant = fugit::TimerInstantU64<TIMER_HZ>;
            type Duration = fugit::TimerDurationU64<TIMER_HZ>;

            const ZERO: Self::Instant = Self::Instant::from_ticks(0);
            const TICK_PERIOD: Self::Duration = Self::Duration::from_ticks(1);

            fn now() -> Self::Instant {
                // Credits to the `time-driver` of `embassy-stm32`.
                // For more info, see the `imxrt` driver.
                fn calc_now(period: u64, counter: $bits) -> u64 {
                    (period << ($bits::BITS - 1)) + u64::from(counter ^ (((period & 1) as $bits) << ($bits::BITS - 1)))
                }

                // Important: period **must** be read first.
                let period = $overflow.load(Ordering::Relaxed);
                compiler_fence(Ordering::Acquire);
                let counter = $timer.cnt().read().cnt();

                Self::Instant::from_ticks(calc_now(period, counter))
            }

            fn set_compare(instant: Self::Instant) {
                let now = Self::now();

                // Since the timer may or may not overflow based on the requested compare val, we check how many ticks are left.
                let val = match instant.checked_duration_since(now) {
                    None => 0, // In the past
                    Some(x) if x.ticks() <= ($bits::MAX as u64) => instant.duration_since_epoch().ticks() as $bits, // Will not overflow
                    Some(_x) => 0, // Will overflow
                };

                $timer.ccr(1).write(|r| r.set_ccr(val));
            }

            fn clear_compare_flag() {
                $timer.sr().modify(|r| r.set_ccif(1, false));
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
                    $timer.sr().modify(|r| r.set_uif(false));
                    $overflow.fetch_add(1, Ordering::Relaxed);
                }
                // Half period
                if $timer.sr().read().ccif(2) {
                    $timer.sr().modify(|r| r.set_ccif(2, false));
                    $overflow.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
    };
}

#[cfg(feature = "stm32_tim2")]
make_timer!(Tim2, TIM2, u32, TIMER2_OVERFLOWS, TIMER2_TQ);

#[cfg(feature = "stm32_tim3")]
make_timer!(Tim3, TIM3, u16, TIMER3_OVERFLOWS, TIMER3_TQ);

#[cfg(feature = "stm32_tim4")]
make_timer!(Tim4, TIM4, u16, TIMER4_OVERFLOWS, TIMER4_TQ);

#[cfg(feature = "stm32_tim5")]
make_timer!(Tim5, TIM5, u16, TIMER5_OVERFLOWS, TIMER5_TQ);

#[cfg(feature = "stm32_tim12")]
make_timer!(Tim12, TIM12, u16, TIMER12_OVERFLOWS, TIMER12_TQ);

#[cfg(feature = "stm32_tim15")]
make_timer!(Tim15, TIM15, u16, TIMER15_OVERFLOWS, TIMER15_TQ);
