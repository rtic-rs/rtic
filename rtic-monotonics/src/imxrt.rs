//! [`Monotonic`] implementations for i.MX RT's GPT peripherals.
//!
//! # Example
//!
//! ```
//! use rtic_monotonics::imxrt::*;
//! use rtic_monotonics::imxrt::Gpt1 as Mono;
//!
//! fn init() {
//!     // Obtain ownership of the timer register block
//!     let gpt1 = unsafe { imxrt_ral::gpt::GPT1::instance() };
//!
//!     // Configure the timer clock source and determine its tick rate
//!     let timer_tickrate_hz = 1_000_000;
//!
//!     // Generate timer token to ensure correct timer interrupt handler is used
//!     let token = rtic_monotonics::create_imxrt_gpt1_token!();
//!
//!     // Start the monotonic
//!     Mono::start(timer_tickrate_hz, gpt1, token);
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
use atomic_polyfill::{AtomicU32, Ordering};
pub use fugit::{self, ExtU64, ExtU64Ceil};
use rtic_time::half_period_counter::calculate_now;

use imxrt_ral as ral;

const TIMER_HZ: u32 = 1_000_000;

#[doc(hidden)]
#[macro_export]
macro_rules! __internal_create_imxrt_timer_interrupt {
    ($mono_timer:ident, $timer:ident, $timer_token:ident) => {{
        #[no_mangle]
        #[allow(non_snake_case)]
        unsafe extern "C" fn $timer() {
            $crate::imxrt::$mono_timer::__tq().on_monotonic_interrupt();
        }

        pub struct $timer_token;

        unsafe impl $crate::InterruptToken<$crate::imxrt::$mono_timer> for $timer_token {}

        $timer_token
    }};
}

/// Register the GPT1 interrupt for the monotonic.
#[cfg(feature = "imxrt_gpt1")]
#[macro_export]
macro_rules! create_imxrt_gpt1_token {
    () => {{
        $crate::__internal_create_imxrt_timer_interrupt!(Gpt1, GPT1, Gpt1Token)
    }};
}

/// Register the GPT2 interrupt for the monotonic.
#[cfg(feature = "imxrt_gpt2")]
#[macro_export]
macro_rules! create_imxrt_gpt2_token {
    () => {{
        $crate::__internal_create_imxrt_timer_interrupt!(Gpt2, GPT2, Gpt2Token)
    }};
}

macro_rules! make_timer {
    ($mono_name:ident, $timer:ident, $period:ident, $tq:ident$(, doc: ($($doc:tt)*))?) => {
        /// Timer implementing [`Monotonic`] which runs at 1 MHz.
        $(
            #[cfg_attr(docsrs, doc(cfg($($doc)*)))]
        )?

        pub struct $mono_name;

        use ral::gpt::$timer;

        /// Number of 2^31 periods elapsed since boot.
        static $period: AtomicU32 = AtomicU32::new(0);
        static $tq: TimerQueue<$mono_name> = TimerQueue::new();

        impl $mono_name {
            /// Starts the monotonic timer.
            ///
            /// - `tick_freq_hz`: The tick frequency of the given timer.
            /// - `gpt`: The GPT timer register block instance.
            /// - `_interrupt_token`: Required for correct timer interrupt handling.
            ///
            /// This method must be called only once.
            pub fn start(tick_freq_hz: u32, gpt: $timer, _interrupt_token: impl crate::InterruptToken<Self>) {
                // Find a prescaler that creates our desired tick frequency
                let previous_prescaler = ral::read_reg!(ral::gpt, gpt, PR, PRESCALER) + 1;
                let previous_clock_freq = tick_freq_hz * previous_prescaler;
                assert!((previous_clock_freq % TIMER_HZ) == 0,
                        "Unable to find a fitting prescaler value!\n    Input: {}/{}\n    Desired: {}",
                        previous_clock_freq, previous_prescaler, TIMER_HZ);
                let prescaler = previous_clock_freq / TIMER_HZ;
                assert!(prescaler > 0);
                assert!(prescaler <= 4096);

                // Disable the timer.
                ral::modify_reg!(ral::gpt, gpt, CR, EN: 0);
                // Clear all status registers.
                ral::write_reg!(ral::gpt, gpt, SR, 0b11_1111);

                // Base configuration
                ral::modify_reg!(ral::gpt, gpt, CR,
                    ENMOD: 1,   // Clear timer state
                    FRR: 1,     // Free-Run mode
                );

                // Reset period
                $period.store(0, Ordering::Relaxed);

                // Prescaler
                ral::modify_reg!(ral::gpt, gpt, PR,
                    PRESCALER: (prescaler - 1), // Scale to our desired clock rate
                );

                // Enable interrupts
                ral::write_reg!(ral::gpt, gpt, IR,
                    ROVIE: 1,   // Rollover interrupt
                    OF1IE: 1,   // Timer compare 1 interrupt (for half-periods)
                    OF2IE: 1,   // Timer compare 2 interrupt (for dynamic wakeup)
                );

                // Configure half-period interrupt
                ral::write_reg!(ral::gpt, gpt, OCR[0], 0x8000_0000);

                // Dynamic interrupt register; for now initialize to zero
                // so it gets combined with rollover interrupt
                ral::write_reg!(ral::gpt, gpt, OCR[1], 0x0000_0000);

                // Enable the timer
                ral::modify_reg!(ral::gpt, gpt, CR, EN: 1);
                ral::modify_reg!(ral::gpt, gpt, CR,
                    ENMOD: 0,   // Keep state when disabled
                );

                $tq.initialize(Self {});

                // SAFETY: We take full ownership of the peripheral and interrupt vector,
                // plus we are not using any external shared resources so we won't impact
                // basepri/source masking based critical sections.
                unsafe {
                    crate::set_monotonic_prio(ral::NVIC_PRIO_BITS, ral::Interrupt::$timer);
                    cortex_m::peripheral::NVIC::unmask(ral::Interrupt::$timer);
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

        rtic_time::embedded_hal_delay_impl_fugit64!($mono_name);

        #[cfg(feature = "embedded-hal-async")]
        rtic_time::embedded_hal_async_delay_impl_fugit64!($mono_name);

        impl Monotonic for $mono_name {
            type Instant = fugit::TimerInstantU64<TIMER_HZ>;
            type Duration = fugit::TimerDurationU64<TIMER_HZ>;

            const ZERO: Self::Instant = Self::Instant::from_ticks(0);
            const TICK_PERIOD: Self::Duration = Self::Duration::from_ticks(1);

            fn now() -> Self::Instant {
                let gpt = unsafe{ $timer::instance() };

                Self::Instant::from_ticks(calculate_now(&$period, || ral::read_reg!(ral::gpt, gpt, CNT)))
            }

            fn set_compare(instant: Self::Instant) {
                let gpt = unsafe{ $timer::instance() };

                // Set the timer regardless of whether it is multiple periods in the future,
                // or even already in the past.
                // The worst thing that can happen is a spurious wakeup, and with a timer
                // period of half an hour, this is hardly a problem.

                let ticks = instant.duration_since_epoch().ticks();
                let ticks_wrapped = ticks as u32;

                ral::write_reg!(ral::gpt, gpt, OCR[1], ticks_wrapped);
            }

            fn clear_compare_flag() {
                let gpt = unsafe{ $timer::instance() };
                ral::write_reg!(ral::gpt, gpt, SR, OF2: 1);
            }

            fn pend_interrupt() {
                cortex_m::peripheral::NVIC::pend(ral::Interrupt::$timer);
            }

            fn on_interrupt() {
                let gpt = unsafe{ $timer::instance() };

                let (rollover, half_rollover) = ral::read_reg!(ral::gpt, gpt, SR, ROV, OF1);

                if rollover != 0 {
                    $period.fetch_add(1, Ordering::Relaxed);
                    ral::write_reg!(ral::gpt, gpt, SR, ROV: 1);
                }

                if half_rollover != 0 {
                    $period.fetch_add(1, Ordering::Relaxed);
                    ral::write_reg!(ral::gpt, gpt, SR, OF1: 1);
                }
            }
        }
    };
}

#[cfg(feature = "imxrt_gpt1")]
make_timer!(Gpt1, GPT1, GPT1_HALFPERIODS, GPT1_TQ);

#[cfg(feature = "imxrt_gpt2")]
make_timer!(Gpt2, GPT2, GPT2_HALFPERIODS, GPT2_TQ);
