//! [`Monotonic`](rtic_time::Monotonic) based on Cortex-M SysTick.
//!
//! Note: this implementation is inefficient as it
//! ticks and generates interrupts at a constant rate.
//!
//! # Example
//!
//! ```
//! use rtic_monotonics::systick::prelude::*;
//!
//! // Create the type `Mono`. It will manage the SysTick timer, and use it to
//! // generate 1000 interrupts per second.
//! systick_monotonic!(Mono, 1_000);
//!
//! fn init() {
//!     let core_peripherals = cortex_m::Peripherals::take().unwrap();
//!     // Start the monotonic using the cortex-m crate's Systick driver.
//!     // We tell it we have a system clock of 12 MHz.
//!     Mono::start(core_peripherals.SYST, 12_000_000);
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
//!
//! By default, SysTick uses [`SystClkSource::Core`] as the clock source.
//! To set the SysTick clock source, use `start_with_clock_source`:
//!
//! ```
//! # use rtic_monotonics::systick::prelude::*;
//! # systick_monotonic!(Mono, 1_000);
//! use rtic_monotonics::systick::SystClkSource;
//!
//! fn init() {
//!     let core_peripherals = cortex_m::Peripherals::take().unwrap();
//!     // Start the monotonic using the cortex-m crate's Systick driver.
//!     // We tell it we have a 12MHz external clock source.
//!     Mono::start_with_clock_source(
//!         core_peripherals.SYST,
//!         12_000_000,
//!         SystClkSource::External,
//!     );
//! }
//! ```

/// Common definitions and traits for using the systick monotonic
pub mod prelude {
    pub use crate::systick_monotonic;

    pub use crate::Monotonic;

    cfg_if::cfg_if! {
        if #[cfg(feature = "systick-64bit")] {
            pub use fugit::{self, ExtU64, ExtU64Ceil};
        } else {
            pub use fugit::{self, ExtU32, ExtU32Ceil};
        }
    }
}

pub use cortex_m::peripheral::{syst::SystClkSource, SYST};

use crate::atomic::Ordering;
use rtic_time::timer_queue::TimerQueue;

use crate::TimerQueueBackend;

cfg_if::cfg_if! {
    if #[cfg(feature = "systick-64bit")] {
        use crate::atomic::AtomicU64;
        static SYSTICK_CNT: AtomicU64 = AtomicU64::new(0);
    } else {
        use crate::atomic::AtomicU32;
        static SYSTICK_CNT: AtomicU32 = AtomicU32::new(0);
    }
}

static SYSTICK_TIMER_QUEUE: TimerQueue<SystickBackend> = TimerQueue::new();

/// Systick based [`TimerQueueBackend`].
pub struct SystickBackend;

impl SystickBackend {
    /// Starts the monotonic timer.
    ///
    /// **Do not use this function directly.**
    ///
    /// Use the prelude macros instead.
    pub fn _start(mut systick: SYST, sysclk: u32, timer_hz: u32, clk_source: SystClkSource) {
        assert!(
            sysclk.is_multiple_of(timer_hz),
            "timer_hz cannot evenly divide sysclk! Please adjust the timer or sysclk frequency."
        );
        let reload = sysclk / timer_hz - 1;

        assert!(reload <= 0x00ff_ffff);
        assert!(reload > 0);

        systick.disable_counter();
        systick.set_clock_source(clk_source);
        systick.set_reload(reload);
        systick.enable_interrupt();
        systick.enable_counter();

        SYSTICK_TIMER_QUEUE.initialize(SystickBackend {});
    }

    fn systick() -> SYST {
        unsafe { core::mem::transmute::<(), SYST>(()) }
    }
}

impl TimerQueueBackend for SystickBackend {
    cfg_if::cfg_if! {
        if #[cfg(feature = "systick-64bit")] {
            type Ticks = u64;
        } else {
            type Ticks = u32;
        }
    }

    fn now() -> Self::Ticks {
        if Self::systick().has_wrapped() {
            SYSTICK_CNT.fetch_add(1, Ordering::AcqRel);
        }

        SYSTICK_CNT.load(Ordering::Relaxed)
    }

    fn set_compare(_: Self::Ticks) {
        // No need to do something here, we get interrupts anyway.
    }

    fn clear_compare_flag() {
        // NOOP with SysTick interrupt
    }

    fn pend_interrupt() {
        cortex_m::peripheral::SCB::set_pendst();
    }

    fn on_interrupt() {
        if Self::systick().has_wrapped() {
            SYSTICK_CNT.fetch_add(1, Ordering::AcqRel);
        }
    }

    fn timer_queue() -> &'static TimerQueue<Self> {
        &SYSTICK_TIMER_QUEUE
    }
}

/// Create a Systick based monotonic and register the Systick interrupt for it.
///
/// This macro expands to produce a new type called `$name`, which has a `fn
/// start()` function for you to call. The type has an implementation of the
/// `rtic_monotonics::TimerQueueBasedMonotonic` trait, the
/// `embedded_hal::delay::DelayNs` trait and the
/// `embedded_hal_async::delay::DelayNs` trait.
///
/// This macro also produces an interrupt handler for the SysTick interrupt, by
/// creating an `extern "C" fn SysTick() { ... }`.
///
/// See [`crate::systick`] for more details.
///
/// # Arguments
///
/// * `name` - The name that the monotonic type will have.
/// * `tick_rate_hz` - The tick rate of the timer peripheral.
///   Can be omitted; defaults to 1kHz.
#[macro_export]
macro_rules! systick_monotonic {
    ($name:ident) => {
        $crate::systick_monotonic!($name, 1_000);
    };
    ($name:ident, $tick_rate_hz:expr) => {
        /// A `Monotonic` based on SysTick.
        pub struct $name;

        impl $name {
            /// Starts the `Monotonic`.
            ///
            /// The `sysclk` parameter is the speed at which SysTick runs at. This value should come from
            /// the clock generation function of the used HAL.
            ///
            /// Panics if it is impossible to achieve the desired monotonic tick rate based
            /// on the given `sysclk` parameter. If that happens, adjust the desired monotonic tick rate.
            ///
            /// This method, or `start_with_clock_source`, must be called only once.
            pub fn start(systick: $crate::systick::SYST, sysclk: u32) {
                Self::start_with_clock_source(systick, sysclk, $crate::systick::SystClkSource::Core)
            }

            /// Starts the `Monotonic` with your preferred SYST clock source.
            ///
            /// The `sysclk` parameter is the speed at which SysTick runs at. This value should come from
            /// the clock generation function of the used HAL, depending on your `clk_source`.
            ///
            /// Panics if it is impossible to achieve the desired monotonic tick rate based
            /// on the given `sysclk` parameter. If that happens, adjust the desired monotonic tick rate.
            ///
            /// This method, or `start`, must be called only once.
            pub fn start_with_clock_source(
                systick: $crate::systick::SYST,
                sysclk: u32,
                clk_source: $crate::systick::SystClkSource,
            ) {
                #[no_mangle]
                #[allow(non_snake_case)]
                unsafe extern "C" fn SysTick() {
                    use $crate::TimerQueueBackend;
                    $crate::systick::SystickBackend::timer_queue().on_monotonic_interrupt();
                }

                $crate::systick::SystickBackend::_start(systick, sysclk, $tick_rate_hz, clk_source);
            }
        }

        impl $crate::TimerQueueBasedMonotonic for $name {
            type Backend = $crate::systick::SystickBackend;
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
