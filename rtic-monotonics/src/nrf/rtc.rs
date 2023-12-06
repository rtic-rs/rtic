//! [`Monotonic`] implementation for the nRF Real Time Clocks (RTC).
//!
//! # Example
//!
//! ```
//! use rtic_monotonics::nrf::rtc::*;
//!
//! fn init() {
//!     # // This is normally provided by the selected PAC
//!     # let rtc = unsafe { core::mem::transmute(()) };
//!     // Generate the required token
//!     let token = rtic_monotonics::create_nrf_rtc0_monotonic_token!();
//!
//!     // Start the monotonic
//!     Rtc0::start(rtc, token);
//! }
//!
//! async fn usage() {
//!     loop {
//!          // Use the monotonic
//!          Rtc0::delay(100.millis()).await;
//!     }
//! }
//! ```

#[cfg(feature = "nrf52810")]
use nrf52810_pac::{self as pac, Interrupt, RTC0, RTC1};
#[cfg(feature = "nrf52811")]
use nrf52811_pac::{self as pac, Interrupt, RTC0, RTC1};
#[cfg(feature = "nrf52832")]
use nrf52832_pac::{self as pac, Interrupt, RTC0, RTC1, RTC2};
#[cfg(feature = "nrf52833")]
use nrf52833_pac::{self as pac, Interrupt, RTC0, RTC1, RTC2};
#[cfg(feature = "nrf52840")]
use nrf52840_pac::{self as pac, Interrupt, RTC0, RTC1, RTC2};
#[cfg(feature = "nrf5340-app")]
use nrf5340_app_pac::{self as pac, Interrupt, RTC0_NS as RTC0, RTC1_NS as RTC1};
#[cfg(feature = "nrf5340-net")]
use nrf5340_net_pac::{self as pac, Interrupt, RTC0_NS as RTC0, RTC1_NS as RTC1};
#[cfg(feature = "nrf9160")]
use nrf9160_pac::{self as pac, Interrupt, RTC0_NS as RTC0, RTC1_NS as RTC1};

use crate::{Monotonic, TimeoutError, TimerQueue};
use atomic_polyfill::{AtomicU32, Ordering};
use core::future::Future;
pub use fugit::{self, ExtU64, ExtU64Ceil};
use rtic_time::half_period_counter::calculate_now;

#[doc(hidden)]
#[macro_export]
macro_rules! __internal_create_nrf_rtc_interrupt {
    ($mono_timer:ident, $rtc:ident, $rtc_token:ident) => {{
        #[no_mangle]
        #[allow(non_snake_case)]
        unsafe extern "C" fn $rtc() {
            $crate::nrf::rtc::$mono_timer::__tq().on_monotonic_interrupt();
        }

        pub struct $rtc_token;

        unsafe impl $crate::InterruptToken<$crate::nrf::rtc::$mono_timer> for $rtc_token {}

        $rtc_token
    }};
}

/// Register the Rtc0 interrupt for the monotonic.
#[macro_export]
macro_rules! create_nrf_rtc0_monotonic_token {
    () => {{
        $crate::__internal_create_nrf_rtc_interrupt!(Rtc0, RTC0, Rtc0Token)
    }};
}

/// Register the Rtc1 interrupt for the monotonic.
#[macro_export]
macro_rules! create_nrf_rtc1_monotonic_token {
    () => {{
        $crate::__internal_create_nrf_rtc_interrupt!(Rtc1, RTC1, Rtc1Token)
    }};
}

/// Register the Rtc2 interrupt for the monotonic.
#[cfg(any(feature = "nrf52832", feature = "nrf52833", feature = "nrf52840"))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "nrf52832", feature = "nrf52833", feature = "nrf52840")))
)]
#[macro_export]
macro_rules! create_nrf_rtc2_monotonic_token {
    () => {{
        $crate::__internal_create_nrf_rtc_interrupt!(Rtc2, RTC2, Rtc2Token)
    }};
}

struct TimerValueU24(u32);
impl rtic_time::half_period_counter::TimerValue for TimerValueU24 {
    const BITS: u32 = 24;
}
impl From<TimerValueU24> for u64 {
    fn from(value: TimerValueU24) -> Self {
        Self::from(value.0)
    }
}

macro_rules! make_rtc {
    ($mono_name:ident, $rtc:ident, $overflow:ident, $tq:ident$(, doc: ($($doc:tt)*))?) => {
        /// Monotonic timer queue implementation.
        $(
            #[cfg_attr(docsrs, doc(cfg($($doc)*)))]
        )?
        pub struct $mono_name;

        static $overflow: AtomicU32 = AtomicU32::new(0);
        static $tq: TimerQueue<$mono_name> = TimerQueue::new();

        impl $mono_name {
            /// Start the timer monotonic.
            pub fn start(rtc: $rtc, _interrupt_token: impl crate::InterruptToken<Self>) {
                unsafe { rtc.prescaler.write(|w| w.bits(0)) };

                // Disable interrupts, as preparation
                rtc.intenclr.write(|w| w
                    .compare0().clear()
                    .compare1().clear()
                    .ovrflw().clear()
                );

                // Configure compare registers
                rtc.cc[0].write(|w| unsafe { w.bits(0) }); // Dynamic wakeup
                rtc.cc[1].write(|w| unsafe { w.bits(0x80_0000) }); // Half-period

                // Timing critical, make sure we don't get interrupted
                critical_section::with(|_|{
                    // Reset the timer
                    rtc.tasks_clear.write(|w| unsafe { w.bits(1) });
                    rtc.tasks_start.write(|w| unsafe { w.bits(1) });

                    // Clear pending events.
                    // Should be close enough to the timer reset that we don't miss any events.
                    rtc.events_ovrflw.write(|w| w);
                    rtc.events_compare[0].write(|w| w);
                    rtc.events_compare[1].write(|w| w);

                    // Make sure overflow counter is synced with the timer value
                    $overflow.store(0, Ordering::SeqCst);

                    // Initialized the timer queue
                    $tq.initialize(Self {});

                    // Enable interrupts.
                    // Should be close enough to the timer reset that we don't miss any events.
                    rtc.intenset.write(|w| w
                        .compare0().set()
                        .compare1().set()
                        .ovrflw().set()
                    );
                    rtc.evtenset.write(|w| w
                        .compare0().set()
                        .compare1().set()
                        .ovrflw().set()
                    );
                });

                // SAFETY: We take full ownership of the peripheral and interrupt vector,
                // plus we are not using any external shared resources so we won't impact
                // basepri/source masking based critical sections.
                unsafe {
                    crate::set_monotonic_prio(pac::NVIC_PRIO_BITS, Interrupt::$rtc);
                    pac::NVIC::unmask(Interrupt::$rtc);
                }
            }

            /// Used to access the underlying timer queue
            #[doc(hidden)]
            pub fn __tq() -> &'static TimerQueue<$mono_name> {
                &$tq
            }

            /// Timeout at a specific time.
            #[inline]
            pub async fn timeout_at<F: Future>(
                instant: <Self as Monotonic>::Instant,
                future: F,
            ) -> Result<F::Output, TimeoutError> {
                $tq.timeout_at(instant, future).await
            }

            /// Timeout after a specific duration.
            #[inline]
            pub async fn timeout_after<F: Future>(
                duration: <Self as Monotonic>::Duration,
                future: F,
            ) -> Result<F::Output, TimeoutError> {
                $tq.timeout_after(duration, future).await
            }

            /// Delay for some duration of time.
            #[inline]
            pub async fn delay(duration: <Self as Monotonic>::Duration) {
                $tq.delay(duration).await;
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
            const ZERO: Self::Instant = Self::Instant::from_ticks(0);
            const TICK_PERIOD: Self::Duration = Self::Duration::from_ticks(1);

            type Instant = fugit::TimerInstantU64<32_768>;
            type Duration = fugit::TimerDurationU64<32_768>;

            fn now() -> Self::Instant {
                let rtc = unsafe { &*$rtc::PTR };
                Self::Instant::from_ticks(calculate_now(
                    || $overflow.load(Ordering::Relaxed),
                    || TimerValueU24(rtc.counter.read().bits())
                ))
            }

            fn on_interrupt() {
                let rtc = unsafe { &*$rtc::PTR };
                if rtc.events_ovrflw.read().bits() == 1 {
                    rtc.events_ovrflw.write(|w| unsafe { w.bits(0) });
                    let prev = $overflow.fetch_add(1, Ordering::Relaxed);
                    assert!(prev % 2 == 1, "Monotonic must have skipped an interrupt!");
                }
                if rtc.events_compare[1].read().bits() == 1 {
                    rtc.events_compare[1].write(|w| unsafe { w.bits(0) });
                    let prev = $overflow.fetch_add(1, Ordering::Relaxed);
                    assert!(prev % 2 == 0, "Monotonic must have skipped an interrupt!");
                }
            }

            // NOTE: To fix errata for RTC, if the release time is within 4 ticks
            // we release as the RTC will not generate a compare interrupt...
            fn should_dequeue_check(release_at: Self::Instant) -> bool {
                Self::now() + <Self as Monotonic>::Duration::from_ticks(4) >= release_at
            }

            fn enable_timer() {}

            fn disable_timer() {}

            fn set_compare(instant: Self::Instant) {
                let rtc = unsafe { &*$rtc::PTR };
                unsafe { rtc.cc[0].write(|w| w.bits(instant.ticks() as u32 & 0xff_ffff)) };
            }

            fn clear_compare_flag() {
                let rtc = unsafe { &*$rtc::PTR };
                unsafe { rtc.events_compare[0].write(|w| w.bits(0)) };
            }

            fn pend_interrupt() {
                pac::NVIC::pend(Interrupt::$rtc);
            }
        }
    };
}

make_rtc!(Rtc0, RTC0, RTC0_OVERFLOWS, RTC0_TQ);
make_rtc!(Rtc1, RTC1, RTC1_OVERFLOWS, RTC1_TQ);
#[cfg(any(feature = "nrf52832", feature = "nrf52833", feature = "nrf52840"))]
make_rtc!(Rtc2, RTC2, RTC2_OVERFLOWS, RTC2_TQ, doc: (any(feature = "nrf52832", feature = "nrf52833", feature = "nrf52840")));
