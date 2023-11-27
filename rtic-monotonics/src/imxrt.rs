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
use atomic_polyfill::{compiler_fence, AtomicU32, Ordering};
pub use fugit::{self, ExtU64, ExtU64Ceil};

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

// Credits to the `time-driver` of `embassy-stm32`.
//
// Clock timekeeping works with something we call "periods", which are time intervals
// of 2^31 ticks. The Clock counter value is 32 bits, so one "overflow cycle" is 2 periods.
//
// A `period` count is maintained in parallel to the Timer hardware `counter`, like this:
// - `period` and `counter` start at 0
// - `period` is incremented on overflow (at counter value 0)
// - `period` is incremented "midway" between overflows (at counter value 0x8000_0000)
//
// Therefore, when `period` is even, counter is in 0..0x7FFF_FFFF. When odd, counter is in 0x8000_0000..0xFFFF_FFFF
// This allows for now() to return the correct value even if it races an overflow.
//
// To get `now()`, `period` is read first, then `counter` is read. If the counter value matches
// the expected range for the `period` parity, we're done. If it doesn't, this means that
// a new period start has raced us between reading `period` and `counter`, so we assume the `counter` value
// corresponds to the next period.
//
// `period` is a 32bit integer, so it overflows on 2^32 * 2^31 / 1_000_000 seconds of uptime, which is 292471 years.
fn calc_now(period: u32, counter: u32) -> u64 {
    (u64::from(period) << 31) + u64::from(counter ^ ((period & 1) << 31))
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

        #[cfg(feature = "embedded-hal-async")]
        impl embedded_hal_async::delay::DelayUs for $mono_name {
            #[inline]
            async fn delay_us(&mut self, us: u32) {
                Self::delay((us as u64).micros_at_least()).await;
            }

            #[inline]
            async fn delay_ms(&mut self, ms: u32) {
                Self::delay((ms as u64).millis_at_least()).await;
            }
        }

        impl embedded_hal::delay::DelayUs for $mono_name {
            fn delay_us(&mut self, us: u32) {
                let done = Self::now() + (us as u64).micros_at_least();
                while Self::now() < done {}
            }
        }

        impl Monotonic for $mono_name {
            type Instant = fugit::TimerInstantU64<TIMER_HZ>;
            type Duration = fugit::TimerDurationU64<TIMER_HZ>;

            const ZERO: Self::Instant = Self::Instant::from_ticks(0);
            const TICK_PERIOD: Self::Duration = Self::Duration::from_ticks(1);

            fn now() -> Self::Instant {
                let gpt = unsafe{ $timer::instance() };

                // Important: period **must** be read first.
                let period = $period.load(Ordering::Relaxed);
                compiler_fence(Ordering::Acquire);
                let counter = ral::read_reg!(ral::gpt, gpt, CNT);

                Self::Instant::from_ticks(calc_now(period, counter))
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
