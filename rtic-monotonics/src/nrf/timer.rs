//! [`Monotonic`] impl for the 32-bit timers of the nRF series.
//!
//! Not all timers are available on all parts. Ensure that only the available
//! timers are exposed by having the correct `nrf52*` feature enabled for `rtic-monotonic`.
//!
//! # Example
//!
//! ```
//! use rtic_monotonics::nrf::timer::*;
//!
//! fn init() {
//!     # // This is normally provided by the selected PAC
//!     # let timer = unsafe { core::mem::transmute(()) };
//!     // Generate the required token
//!     let token = rtic_monotonics::create_nrf_timer0_monotonic_token!();
//!
//!     // Start the monotonic
//!     Timer0::start(timer, token);
//! }
//!
//! async fn usage() {
//!     loop {
//!          // Use the monotonic
//!          Timer0::delay(100.millis()).await;
//!     }
//! }
//! ```

use crate::{Monotonic, TimeoutError, TimerQueue};
use atomic_polyfill::{AtomicU32, Ordering};
use core::future::Future;
pub use fugit::{self, ExtU64};

#[cfg(feature = "nrf52810")]
use nrf52810_pac::{self as pac, Interrupt, TIMER0, TIMER1, TIMER2};
#[cfg(feature = "nrf52811")]
use nrf52811_pac::{self as pac, Interrupt, TIMER0, TIMER1, TIMER2};
#[cfg(feature = "nrf52832")]
use nrf52832_pac::{self as pac, Interrupt, TIMER0, TIMER1, TIMER2, TIMER3, TIMER4};
#[cfg(feature = "nrf52833")]
use nrf52833_pac::{self as pac, Interrupt, TIMER0, TIMER1, TIMER2, TIMER3, TIMER4};
#[cfg(feature = "nrf52840")]
use nrf52840_pac::{self as pac, Interrupt, TIMER0, TIMER1, TIMER2, TIMER3, TIMER4};
#[cfg(feature = "nrf5340-app")]
use nrf5340_app_pac::{
    self as pac, Interrupt, TIMER0_NS as TIMER0, TIMER1_NS as TIMER1, TIMER2_NS as TIMER2,
};
#[cfg(feature = "nrf5340-net")]
use nrf5340_net_pac::{
    self as pac, Interrupt, TIMER0_NS as TIMER0, TIMER1_NS as TIMER1, TIMER2_NS as TIMER2,
};
#[cfg(feature = "nrf9160")]
use nrf9160_pac::{
    self as pac, Interrupt, TIMER0_NS as TIMER0, TIMER1_NS as TIMER1, TIMER2_NS as TIMER2,
};

#[doc(hidden)]
#[macro_export]
macro_rules! __internal_create_nrf_timer_interrupt {
    ($mono_timer:ident, $timer:ident, $timer_token:ident) => {{
        #[no_mangle]
        #[allow(non_snake_case)]
        unsafe extern "C" fn $timer() {
            $crate::nrf::timer::$mono_timer::__tq().on_monotonic_interrupt();
        }

        pub struct $timer_token;

        unsafe impl $crate::InterruptToken<$crate::nrf::timer::$mono_timer> for $timer_token {}

        $timer_token
    }};
}

/// Register the Timer0 interrupt for the monotonic.
#[macro_export]
macro_rules! create_nrf_timer0_monotonic_token {
    () => {{
        $crate::__internal_create_nrf_timer_interrupt!(Timer0, TIMER0, Timer0Token)
    }};
}

/// Register the Timer1 interrupt for the monotonic.
#[macro_export]
macro_rules! create_nrf_timer1_monotonic_token {
    () => {{
        $crate::__internal_create_nrf_timer_interrupt!(Timer1, TIMER1, Timer1Token)
    }};
}

/// Register the Timer2 interrupt for the monotonic.
#[macro_export]
macro_rules! create_nrf_timer2_monotonic_token {
    () => {{
        $crate::__internal_create_nrf_timer_interrupt!(Timer2, TIMER2, Timer2Token)
    }};
}

/// Register the Timer3 interrupt for the monotonic.
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "nrf52832", feature = "nrf52833", feature = "nrf52840")))
)]
#[cfg(any(feature = "nrf52832", feature = "nrf52833", feature = "nrf52840"))]
#[macro_export]
macro_rules! create_nrf_timer3_monotonic_token {
    () => {{
        $crate::__internal_create_nrf_timer_interrupt!(Timer3, TIMER3, Timer3Token)
    }};
}

/// Register the Timer4 interrupt for the monotonic.
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "nrf52832", feature = "nrf52833", feature = "nrf52840")))
)]
#[cfg(any(feature = "nrf52832", feature = "nrf52833", feature = "nrf52840"))]
#[macro_export]
macro_rules! create_nrf_timer4_monotonic_token {
    () => {{
        $crate::__internal_create_nrf_timer_interrupt!(Timer4, TIMER4, Timer4Token)
    }};
}

macro_rules! make_timer {
    ($mono_name:ident, $timer:ident, $overflow:ident, $tq:ident$(, doc: ($($doc:tt)*))?) => {
        /// Monotonic timer queue implementation.
        $(
            #[cfg_attr(docsrs, doc(cfg($($doc)*)))]
        )?
        pub struct $mono_name;

        static $overflow: AtomicU32 = AtomicU32::new(0);
        static $tq: TimerQueue<$mono_name> = TimerQueue::new();

        impl $mono_name {
            /// Start the timer monotonic.
            pub fn start(timer: $timer, _interrupt_token: impl crate::InterruptToken<Self>) {
                // 1 MHz
                timer.prescaler.write(|w| unsafe { w.prescaler().bits(4) });
                timer.bitmode.write(|w| w.bitmode()._32bit());
                timer
                    .intenset
                    .modify(|_, w| w.compare0().set().compare1().set());
                timer.cc[1].write(|w| unsafe { w.cc().bits(0) }); // Overflow
                timer.tasks_clear.write(|w| unsafe { w.bits(1) });
                timer.tasks_start.write(|w| unsafe { w.bits(1) });

                $tq.initialize(Self {});

                timer.events_compare[0].write(|w| w);
                timer.events_compare[1].write(|w| w);

                // SAFETY: We take full ownership of the peripheral and interrupt vector,
                // plus we are not using any external shared resources so we won't impact
                // basepri/source masking based critical sections.
                unsafe {
                    crate::set_monotonic_prio(pac::NVIC_PRIO_BITS, Interrupt::$timer);
                    pac::NVIC::unmask(Interrupt::$timer);
                }
            }

            /// Used to access the underlying timer queue
            #[doc(hidden)]
            pub fn __tq() -> &'static TimerQueue<$mono_name> {
                &$tq
            }

            #[inline(always)]
            fn is_overflow() -> bool {
                let timer = unsafe { &*$timer::PTR };
                timer.events_compare[1].read().bits() & 1 != 0
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

        impl embedded_hal::delay::DelayUs for $mono_name {
            fn delay_us(&mut self, us: u32) {
                let done = Self::now() + (us as u64).micros();
                while Self::now() < done {}
            }
        }

        impl Monotonic for $mono_name {
            const ZERO: Self::Instant = Self::Instant::from_ticks(0);

            type Instant = fugit::TimerInstantU64<1_000_000>;
            type Duration = fugit::TimerDurationU64<1_000_000>;

            fn now() -> Self::Instant {
                // In a critical section to not get a race between overflow updates and reading it
                // and the flag here.
                critical_section::with(|_| {
                    let timer = unsafe { &*$timer::PTR };
                    timer.tasks_capture[2].write(|w| unsafe { w.bits(1) });
                    let cnt = timer.cc[2].read().bits();

                    let unhandled_overflow = if Self::is_overflow() {
                        // The overflow has not been handled yet, so add an extra to the read overflow.
                        1
                    } else {
                        0
                    };

                    timer.tasks_capture[2].write(|w| unsafe { w.bits(1) });
                    let new_cnt = timer.cc[2].read().bits();
                    let cnt = if new_cnt >= cnt { cnt } else { new_cnt } as u64;

                    Self::Instant::from_ticks(
                        (unhandled_overflow + $overflow.load(Ordering::Relaxed) as u64) << 32
                            | cnt as u64,
                    )
                })
            }

            fn on_interrupt() {
                let timer = unsafe { &*$timer::PTR };

                // If there is a compare match on channel 1, it is an overflow
                if Self::is_overflow() {
                    timer.events_compare[1].write(|w| w);
                    $overflow.fetch_add(1, Ordering::SeqCst);
                }
            }

            fn enable_timer() {}

            fn disable_timer() {}

            fn set_compare(instant: Self::Instant) {
                let timer = unsafe { &*$timer::PTR };
                timer.cc[0].write(|w| unsafe { w.cc().bits(instant.ticks() as u32) });
            }

            fn clear_compare_flag() {
                let timer = unsafe { &*$timer::PTR };
                timer.events_compare[0].write(|w| w);
            }

            fn pend_interrupt() {
                pac::NVIC::pend(Interrupt::$timer);
            }
        }
    };
}

make_timer!(Timer0, TIMER0, TIMER0_OVERFLOWS, TIMER0_TQ);
make_timer!(Timer1, TIMER1, TIMER1_OVERFLOWS, TIMER1_TQ);
make_timer!(Timer2, TIMER2, TIMER2_OVERFLOWS, TIMER2_TQ);
#[cfg(any(feature = "nrf52832", feature = "nrf52833", feature = "nrf52840"))]
make_timer!(Timer3, TIMER3, TIMER3_OVERFLOWS, TIMER3_TQ, doc: (any(feature = "nrf52832", feature = "nrf52833", feature = "nrf52840")));
#[cfg(any(feature = "nrf52832", feature = "nrf52833", feature = "nrf52840"))]
make_timer!(Timer4, TIMER4, TIMER4_OVERFLOWS, TIMER4_TQ, doc: (any(feature = "nrf52832", feature = "nrf52833", feature = "nrf52840")));
