//! [`Monotonic`](rtic_time::Monotonic) implementation for the 32-bit timers of the nRF series.
//!
//! Not all timers are available on all parts. Ensure that only the available
//! timers are exposed by having the correct `nrf52*` feature enabled for `rtic-monotonics`.
//!
//! # Example
//!
//! ```
//! use rtic_monotonics::nrf::timer::prelude::*;
//!
//! // Create the type `Mono`. It will manage the TIMER0 timer, and
//! // run with a resolution of 1 µs (1,000,000 ticks per second).
//! nrf_timer0_monotonic!(Mono, 1_000_000);
//!
//! fn init() {
//!     # // This is normally provided by the selected PAC
//!     # let TIMER0 = unsafe { core::mem::transmute(()) };
//!     // Start the monotonic, passing ownership of a TIMER0 object from the
//!     // relevant nRF52x PAC.
//!     Mono::start(TIMER0);
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

/// Common definitions and traits for using the nRF Timer monotonics
pub mod prelude {
    pub use crate::nrf_timer0_monotonic;
    pub use crate::nrf_timer1_monotonic;
    pub use crate::nrf_timer2_monotonic;
    #[cfg(any(feature = "nrf52832", feature = "nrf52833", feature = "nrf52840"))]
    pub use crate::nrf_timer3_monotonic;
    #[cfg(any(feature = "nrf52832", feature = "nrf52833", feature = "nrf52840"))]
    pub use crate::nrf_timer4_monotonic;

    pub use crate::Monotonic;
    pub use fugit::{self, ExtU64, ExtU64Ceil};
}

#[cfg(feature = "nrf52805")]
#[doc(hidden)]
pub use nrf52805_pac::{self as pac, TIMER0, TIMER1, TIMER2};
#[cfg(feature = "nrf52810")]
#[doc(hidden)]
pub use nrf52810_pac::{self as pac, TIMER0, TIMER1, TIMER2};
#[cfg(feature = "nrf52811")]
#[doc(hidden)]
pub use nrf52811_pac::{self as pac, TIMER0, TIMER1, TIMER2};
#[cfg(feature = "nrf52832")]
#[doc(hidden)]
pub use nrf52832_pac::{self as pac, TIMER0, TIMER1, TIMER2, TIMER3, TIMER4};
#[cfg(feature = "nrf52833")]
#[doc(hidden)]
pub use nrf52833_pac::{self as pac, TIMER0, TIMER1, TIMER2, TIMER3, TIMER4};
#[cfg(feature = "nrf52840")]
#[doc(hidden)]
pub use nrf52840_pac::{self as pac, TIMER0, TIMER1, TIMER2, TIMER3, TIMER4};
#[cfg(feature = "nrf5340-app")]
#[doc(hidden)]
pub use nrf5340_app_pac::{
    self as pac, TIMER0_NS as TIMER0, TIMER1_NS as TIMER1, TIMER2_NS as TIMER2,
};
#[cfg(feature = "nrf5340-net")]
#[doc(hidden)]
pub use nrf5340_net_pac::{
    self as pac, TIMER0_NS as TIMER0, TIMER1_NS as TIMER1, TIMER2_NS as TIMER2,
};
#[cfg(feature = "nrf9160")]
#[doc(hidden)]
pub use nrf9160_pac::{self as pac, TIMER0_NS as TIMER0, TIMER1_NS as TIMER1, TIMER2_NS as TIMER2};

use crate::atomic::{AtomicU32, Ordering};
use rtic_time::{
    half_period_counter::calculate_now,
    timer_queue::{TimerQueue, TimerQueueBackend},
};

#[doc(hidden)]
#[macro_export]
macro_rules! __internal_create_nrf_timer_interrupt {
    ($mono_backend:ident, $timer:ident) => {
        #[no_mangle]
        #[allow(non_snake_case)]
        unsafe extern "C" fn $timer() {
            use $crate::TimerQueueBackend;
            $crate::nrf::timer::$mono_backend::timer_queue().on_monotonic_interrupt();
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __internal_create_nrf_timer_struct {
    ($name:ident, $mono_backend:ident, $timer:ident, $tick_rate_hz:expr) => {
        /// A `Monotonic` based on the nRF Timer peripheral.
        pub struct $name;

        impl $name {
            /// Starts the `Monotonic`.
            ///
            /// This method must be called only once.
            pub fn start(timer: $crate::nrf::timer::$timer) {
                $crate::__internal_create_nrf_timer_interrupt!($mono_backend, $timer);

                const PRESCALER: u8 = match $tick_rate_hz {
                    16_000_000 => 0,
                    8_000_000 => 1,
                    4_000_000 => 2,
                    2_000_000 => 3,
                    1_000_000 => 4,
                    500_000 => 5,
                    250_000 => 6,
                    125_000 => 7,
                    62_500 => 8,
                    31_250 => 9,
                    _ => panic!("Timer cannot run at desired tick rate!"),
                };

                $crate::nrf::timer::$mono_backend::_start(timer, PRESCALER);
            }
        }

        impl $crate::TimerQueueBasedMonotonic for $name {
            type Backend = $crate::nrf::timer::$mono_backend;
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

/// Create an Timer0 based monotonic and register the TIMER0 interrupt for it.
///
/// See [`crate::nrf::timer`] for more details.
#[macro_export]
macro_rules! nrf_timer0_monotonic {
    ($name:ident, $tick_rate_hz:expr) => {
        $crate::__internal_create_nrf_timer_struct!($name, Timer0Backend, TIMER0, $tick_rate_hz);
    };
}

/// Create an Timer1 based monotonic and register the TIMER1 interrupt for it.
///
/// See [`crate::nrf::timer`] for more details.
#[macro_export]
macro_rules! nrf_timer1_monotonic {
    ($name:ident, $tick_rate_hz:expr) => {
        $crate::__internal_create_nrf_timer_struct!($name, Timer1Backend, TIMER1, $tick_rate_hz);
    };
}

/// Create an Timer2 based monotonic and register the TIMER2 interrupt for it.
///
/// See [`crate::nrf::timer`] for more details.
#[macro_export]
macro_rules! nrf_timer2_monotonic {
    ($name:ident, $tick_rate_hz:expr) => {
        $crate::__internal_create_nrf_timer_struct!($name, Timer2Backend, TIMER2, $tick_rate_hz);
    };
}

/// Create an Timer3 based monotonic and register the TIMER3 interrupt for it.
///
/// See [`crate::nrf::timer`] for more details.
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "nrf52832", feature = "nrf52833", feature = "nrf52840")))
)]
#[cfg(any(feature = "nrf52832", feature = "nrf52833", feature = "nrf52840"))]
#[macro_export]
macro_rules! nrf_timer3_monotonic {
    ($name:ident, $tick_rate_hz:expr) => {
        $crate::__internal_create_nrf_timer_struct!($name, Timer3Backend, TIMER3, $tick_rate_hz);
    };
}

/// Create an Timer4 based monotonic and register the TIMER4 interrupt for it.
///
/// See [`crate::nrf::timer`] for more details.
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "nrf52832", feature = "nrf52833", feature = "nrf52840")))
)]
#[cfg(any(feature = "nrf52832", feature = "nrf52833", feature = "nrf52840"))]
#[macro_export]
macro_rules! nrf_timer4_monotonic {
    ($name:ident, $tick_rate_hz:expr) => {
        $crate::__internal_create_nrf_timer_struct!($name, Timer4Backend, TIMER4, $tick_rate_hz);
    };
}

macro_rules! make_timer {
    ($backend_name:ident, $timer:ident, $overflow:ident, $tq:ident$(, doc: ($($doc:tt)*))?) => {
        /// Timer peripheral based [`TimerQueueBackend`].
        $(
            #[cfg_attr(docsrs, doc(cfg($($doc)*)))]
        )?
        pub struct $backend_name;

        static $overflow: AtomicU32 = AtomicU32::new(0);
        static $tq: TimerQueue<$backend_name> = TimerQueue::new();

        impl $backend_name {
            /// Starts the timer.
            ///
            /// **Do not use this function directly.**
            ///
            /// Use the prelude macros instead.
            pub fn _start(timer: $timer, prescaler: u8) {
                timer.prescaler.write(|w| unsafe { w.prescaler().bits(prescaler) });
                timer.bitmode.write(|w| w.bitmode()._32bit());

                // Disable interrupts, as preparation
                timer.intenclr.modify(|_, w| w
                    .compare0().clear()
                    .compare1().clear()
                    .compare2().clear()
                );

                // Configure compare registers
                timer.cc[0].write(|w| unsafe { w.cc().bits(0) }); // Dynamic wakeup
                timer.cc[1].write(|w| unsafe { w.cc().bits(0x0000_0000) }); // Overflow
                timer.cc[2].write(|w| unsafe { w.cc().bits(0x8000_0000) }); // Half-period

                // Timing critical, make sure we don't get interrupted
                critical_section::with(|_|{
                    // Reset the timer
                    timer.tasks_clear.write(|w| unsafe { w.bits(1) });
                    timer.tasks_start.write(|w| unsafe { w.bits(1) });

                    // Clear pending events.
                    // Should be close enough to the timer reset that we don't miss any events.
                    timer.events_compare[0].write(|w| w);
                    timer.events_compare[1].write(|w| w);
                    timer.events_compare[2].write(|w| w);

                    // Make sure overflow counter is synced with the timer value
                    $overflow.store(0, Ordering::SeqCst);

                    // Initialized the timer queue
                    $tq.initialize(Self {});

                    // Enable interrupts.
                    // Should be close enough to the timer reset that we don't miss any events.
                    timer.intenset.modify(|_, w| w
                        .compare0().set()
                        .compare1().set()
                        .compare2().set()
                    );
                });

                // SAFETY: We take full ownership of the peripheral and interrupt vector,
                // plus we are not using any external shared resources so we won't impact
                // basepri/source masking based critical sections.
                unsafe {
                    crate::set_monotonic_prio(pac::NVIC_PRIO_BITS, pac::Interrupt::$timer);
                    pac::NVIC::unmask(pac::Interrupt::$timer);
                }
            }
        }

        impl TimerQueueBackend for $backend_name {
            type Ticks = u64;

            fn now() -> Self::Ticks {
                let timer = unsafe { &*$timer::PTR };

                calculate_now(
                    || $overflow.load(Ordering::Relaxed),
                    || {
                        timer.tasks_capture[3].write(|w| unsafe { w.bits(1) });
                        timer.cc[3].read().bits()
                    }
                )
            }

            fn on_interrupt() {
                let timer = unsafe { &*$timer::PTR };

                // If there is a compare match on channel 1, it is an overflow
                if timer.events_compare[1].read().bits() & 1 != 0 {
                    timer.events_compare[1].write(|w| w);
                    let prev = $overflow.fetch_add(1, Ordering::Relaxed);
                    assert!(prev % 2 == 1, "Monotonic must have skipped an interrupt!");
                }

                // If there is a compare match on channel 2, it is a half-period overflow
                if timer.events_compare[2].read().bits() & 1 != 0 {
                    timer.events_compare[2].write(|w| w);
                    let prev = $overflow.fetch_add(1, Ordering::Relaxed);
                    assert!(prev % 2 == 0, "Monotonic must have skipped an interrupt!");
                }
            }

            fn set_compare(instant: Self::Ticks) {
                let timer = unsafe { &*$timer::PTR };
                timer.cc[0].write(|w| unsafe { w.cc().bits(instant as u32) });
            }

            fn clear_compare_flag() {
                let timer = unsafe { &*$timer::PTR };
                timer.events_compare[0].write(|w| w);
            }

            fn pend_interrupt() {
                pac::NVIC::pend(pac::Interrupt::$timer);
            }

            fn timer_queue() -> &'static TimerQueue<$backend_name> {
                &$tq
            }
        }
    };
}

make_timer!(Timer0Backend, TIMER0, TIMER0_OVERFLOWS, TIMER0_TQ);
make_timer!(Timer1Backend, TIMER1, TIMER1_OVERFLOWS, TIMER1_TQ);
make_timer!(Timer2Backend, TIMER2, TIMER2_OVERFLOWS, TIMER2_TQ);
#[cfg(any(feature = "nrf52832", feature = "nrf52833", feature = "nrf52840"))]
make_timer!(Timer3Backend, TIMER3, TIMER3_OVERFLOWS, TIMER3_TQ, doc: (any(feature = "nrf52832", feature = "nrf52833", feature = "nrf52840")));
#[cfg(any(feature = "nrf52832", feature = "nrf52833", feature = "nrf52840"))]
make_timer!(Timer4Backend, TIMER4, TIMER4_OVERFLOWS, TIMER4_TQ, doc: (any(feature = "nrf52832", feature = "nrf52833", feature = "nrf52840")));
