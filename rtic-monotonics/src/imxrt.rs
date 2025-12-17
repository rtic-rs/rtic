//! [`Monotonic`](rtic_time::Monotonic) implementations for i.MX RT's GPT peripherals.
//!
//! # Example
//!
//! ```
//! use rtic_monotonics::imxrt::prelude::*;
//!
//! // Create the type `Mono`. It will manage the GPT1 timer, and
//! // run with a resolution of 1 µs (1,000,000 ticks per second).
//! imxrt_gpt1_monotonic!(Mono, 1_000_000);
//!
//! fn init() {
//!     // Obtain ownership of the timer register block.
//!     let gpt1 = unsafe { imxrt_ral::gpt::GPT1::instance() };
//!
//!     // Configure the timer tick rate as specified earlier
//!     todo!("Configure the gpt1 peripheral to a tick rate of 1_000_000");
//!
//!     // Start the monotonic
//!     Mono::start(gpt1);
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

use crate::atomic::{AtomicU32, Ordering};
use rtic_time::{
    half_period_counter::calculate_now,
    timer_queue::{TimerQueue, TimerQueueBackend},
};

pub use imxrt_ral as ral;

/// Common definitions and traits for using the i.MX RT monotonics
pub mod prelude {
    #[cfg(feature = "imxrt_gpt1")]
    pub use crate::imxrt_gpt1_monotonic;
    #[cfg(feature = "imxrt_gpt2")]
    pub use crate::imxrt_gpt2_monotonic;

    pub use crate::Monotonic;
    pub use fugit::{self, ExtU64, ExtU64Ceil};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __internal_create_imxrt_timer_interrupt {
    ($mono_backend:ident, $timer:ident) => {
        #[no_mangle]
        #[allow(non_snake_case)]
        unsafe extern "C" fn $timer() {
            use $crate::TimerQueueBackend;
            $crate::imxrt::$mono_backend::timer_queue().on_monotonic_interrupt();
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __internal_create_imxrt_timer_struct {
    ($name:ident, $mono_backend:ident, $timer:ident, $tick_rate_hz:expr) => {
        /// A `Monotonic` based on the GPT peripheral.
        pub struct $name;

        impl $name {
            /// Starts the `Monotonic`.
            ///
            /// This method must be called only once.
            pub fn start(gpt: $crate::imxrt::ral::gpt::$timer) {
                $crate::__internal_create_imxrt_timer_interrupt!($mono_backend, $timer);

                $crate::imxrt::$mono_backend::_start(gpt);
            }
        }

        impl $crate::TimerQueueBasedMonotonic for $name {
            type Backend = $crate::imxrt::$mono_backend;
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

/// Create a GPT1 based monotonic and register the GPT1 interrupt for it.
///
/// See [`crate::imxrt`] for more details.
///
/// # Arguments
///
/// * `name` - The name that the monotonic type will have.
/// * `tick_rate_hz` - The tick rate of the timer peripheral. It's the user's responsibility
///   to configure the peripheral to the given frequency before starting the
///   monotonic.
#[cfg(feature = "imxrt_gpt1")]
#[macro_export]
macro_rules! imxrt_gpt1_monotonic {
    ($name:ident, $tick_rate_hz:expr) => {
        $crate::__internal_create_imxrt_timer_struct!($name, Gpt1Backend, GPT1, $tick_rate_hz);
    };
}

/// Create a GPT2 based monotonic and register the GPT2 interrupt for it.
///
/// See [`crate::imxrt`] for more details.
///
/// # Arguments
///
/// * `name` - The name that the monotonic type will have.
/// * `tick_rate_hz` - The tick rate of the timer peripheral. It's the user's responsibility
///   to configure the peripheral to the given frequency before starting the
///   monotonic.
#[cfg(feature = "imxrt_gpt2")]
#[macro_export]
macro_rules! imxrt_gpt2_monotonic {
    ($name:ident, $tick_rate_hz:expr) => {
        $crate::__internal_create_imxrt_timer_struct!($name, Gpt2Backend, GPT2, $tick_rate_hz);
    };
}

macro_rules! make_timer {
    ($mono_name:ident, $backend_name:ident, $timer:ident, $period:ident, $tq:ident$(, doc: ($($doc:tt)*))?) => {
        /// GPT based [`TimerQueueBackend`].
        $(
            #[cfg_attr(docsrs, doc(cfg($($doc)*)))]
        )?

        pub struct $backend_name;

        use ral::gpt::$timer;

        /// Number of 2^31 periods elapsed since boot.
        static $period: AtomicU32 = AtomicU32::new(0);
        static $tq: TimerQueue<$backend_name> = TimerQueue::new();

        impl $backend_name {
            /// Starts the timer.
            ///
            /// **Do not use this function directly.**
            ///
            /// Use the prelude macros instead.
            pub fn _start(gpt: $timer) {

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
                $period.store(0, Ordering::SeqCst);

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

                // Initialize timer queue
                $tq.initialize(Self {});

                // Enable the timer
                ral::modify_reg!(ral::gpt, gpt, CR, EN: 1);
                ral::modify_reg!(ral::gpt, gpt, CR,
                    ENMOD: 0,   // Keep state when disabled
                );

                // SAFETY: We take full ownership of the peripheral and interrupt vector,
                // plus we are not using any external shared resources so we won't impact
                // basepri/source masking based critical sections.
                unsafe {
                    crate::set_monotonic_prio(ral::NVIC_PRIO_BITS, ral::Interrupt::$timer);
                    cortex_m::peripheral::NVIC::unmask(ral::Interrupt::$timer);
                }
            }
        }

        impl TimerQueueBackend for $backend_name {
            type Ticks = u64;

            fn now() -> Self::Ticks {
                let gpt = unsafe{ $timer::instance() };

                calculate_now(
                    || $period.load(Ordering::Relaxed),
                    || ral::read_reg!(ral::gpt, gpt, CNT)
                )
            }

            fn set_compare(instant: Self::Ticks) {
                let gpt = unsafe{ $timer::instance() };

                // Set the timer regardless of whether it is multiple periods in the future,
                // or even already in the past.
                // The worst thing that can happen is a spurious wakeup, and with a timer
                // period of half an hour, this is hardly a problem.

                let ticks_wrapped = instant as u32;

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
                    let prev = $period.fetch_add(1, Ordering::Relaxed);
                    ral::write_reg!(ral::gpt, gpt, SR, ROV: 1);
                    assert!(prev % 2 == 1, "Monotonic must have skipped an interrupt!");
                }

                if half_rollover != 0 {
                    let prev = $period.fetch_add(1, Ordering::Relaxed);
                    ral::write_reg!(ral::gpt, gpt, SR, OF1: 1);
                    assert!(prev % 2 == 0, "Monotonic must have skipped an interrupt!");
                }
            }

            fn timer_queue() -> &'static TimerQueue<Self> {
                &$tq
            }
        }
    };
}

#[cfg(feature = "imxrt_gpt1")]
make_timer!(Gpt1, Gpt1Backend, GPT1, GPT1_HALFPERIODS, GPT1_TQ);

#[cfg(feature = "imxrt_gpt2")]
make_timer!(Gpt2, Gpt2Backend, GPT2, GPT2_HALFPERIODS, GPT2_TQ);
