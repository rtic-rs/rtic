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
pub use fugit::{self, ExtU64};

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
                rtc.intenset.write(|w| w.compare0().set().ovrflw().set());
                rtc.evtenset.write(|w| w.compare0().set().ovrflw().set());

                rtc.tasks_clear.write(|w| unsafe { w.bits(1) });
                rtc.tasks_start.write(|w| unsafe { w.bits(1) });

                $tq.initialize(Self {});

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

            #[inline(always)]
            fn is_overflow() -> bool {
                let rtc = unsafe { &*$rtc::PTR };
                rtc.events_ovrflw.read().bits() == 1
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

        #[cfg(feature = "embedded-hal-async")]
        impl embedded_hal_async::delay::DelayUs for $mono_name {
            #[inline]
            async fn delay_us(&mut self, us: u32) {
               Self::delay((us as u64).micros()).await;
            }

            #[inline]
            async fn delay_ms(&mut self, ms: u32) {
                Self::delay((ms as u64).millis()).await;
            }
        }

        impl Monotonic for $mono_name {
            const ZERO: Self::Instant = Self::Instant::from_ticks(0);

            type Instant = fugit::TimerInstantU64<32_768>;
            type Duration = fugit::TimerDurationU64<32_768>;

            fn now() -> Self::Instant {
                // In a critical section to not get a race between overflow updates and reading it
                // and the flag here.
                critical_section::with(|_| {
                    let rtc = unsafe { &*$rtc::PTR };
                    let cnt = rtc.counter.read().bits();
                    // OVERFLOW HAPPENS HERE race needs to be handled
                    let ovf = if Self::is_overflow() {
                        $overflow.load(Ordering::Relaxed) + 1
                    } else {
                        $overflow.load(Ordering::Relaxed)
                    } as u64;

                    // Check and fix if above race happened
                    let new_cnt = rtc.counter.read().bits();
                    let cnt = if new_cnt >= cnt { cnt } else { new_cnt } as u64;

                    Self::Instant::from_ticks((ovf << 24) | cnt)
                })
            }

            fn on_interrupt() {
                let rtc = unsafe { &*$rtc::PTR };
                if Self::is_overflow() {
                    $overflow.fetch_add(1, Ordering::SeqCst);
                    rtc.events_ovrflw.write(|w| unsafe { w.bits(0) });
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
                unsafe { rtc.cc[0].write(|w| w.bits(instant.ticks() as u32 & 0xffffff)) };
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
