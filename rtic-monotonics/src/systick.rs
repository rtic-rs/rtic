//! [`Monotonic`] based on Cortex-M SysTick. Note: this implementation is inefficient as it
//! ticks and generates interrupts at a constant rate.
//!
//! Currently, the following tick rates are supported:
//!
//! | Feature          | Tick rate | Precision |
//! |:----------------:|----------:|----------:|
//! | (none / default) |  1 kHz    |      1 ms |
//! |   systick-100hz  | 100 Hz    |     10 ms |
//! |   systick-10khz  | 10 kHz    |    0.1 ms |

//! # Example
//!
//! ```
//! use rtic_monotonics::systick::*;
//!
//! fn init() {
//!     # // This is normally provided by the selected PAC
//!     # let systick = unsafe { core::mem::transmute(()) };
//!     // Generate the required token
//!     let systick_token = rtic_monotonics::create_systick_token!();
//!
//!     // Start the monotonic
//!     Systick::start(systick, 12_000_000, systick_token);
//! }
//!
//! async fn usage() {
//!     loop {
//!          // Use the monotonic
//!          Systick::delay(100.millis()).await;
//!     }
//! }
//! ```

use super::Monotonic;
pub use super::{TimeoutError, TimerQueue};
use atomic_polyfill::Ordering;
use cortex_m::peripheral::SYST;
pub use fugit;
cfg_if::cfg_if! {
    if #[cfg(feature = "systick-64bit")] {
        pub use fugit::{ExtU64, ExtU64Ceil};
        use atomic_polyfill::AtomicU64;
        static SYSTICK_CNT: AtomicU64 = AtomicU64::new(0);
    } else {
        pub use fugit::{ExtU32, ExtU32Ceil};
        use atomic_polyfill::AtomicU32;
        static SYSTICK_CNT: AtomicU32 = AtomicU32::new(0);
    }
}
static SYSTICK_TIMER_QUEUE: TimerQueue<Systick> = TimerQueue::new();

// Features should be additive, here systick-100hz gets picked if both
// `systick-100hz` and `systick-10khz` are enabled.

cfg_if::cfg_if! {
    if #[cfg(feature = "systick-100hz")]
    {
        const TIMER_HZ: u32 = 100;
    } else if #[cfg(feature = "systick-10khz")]
    {
        const TIMER_HZ: u32 = 10_000;
    } else {
        // Default case is 1 kHz
        const TIMER_HZ: u32 = 1_000;
    }
}

/// Systick implementing [`Monotonic`] which runs at 1 kHz, 100Hz or 10 kHz.
pub struct Systick;

impl Systick {
    /// Start a `Monotonic` based on SysTick.
    ///
    /// The `sysclk` parameter is the speed at which SysTick runs at. This value should come from
    /// the clock generation function of the used HAL.
    ///
    /// Notice that the actual rate of the timer is a best approximation based on the given
    /// `sysclk` and `TIMER_HZ`.
    ///
    /// Note: Give the return value to `TimerQueue::initialize()` to initialize the timer queue.
    pub fn start(
        mut systick: cortex_m::peripheral::SYST,
        sysclk: u32,
        _interrupt_token: impl crate::InterruptToken<Self>,
    ) {
        // + TIMER_HZ / 2 provides round to nearest instead of round to 0.
        // - 1 as the counter range is inclusive [0, reload]
        let reload = (sysclk + TIMER_HZ / 2) / TIMER_HZ - 1;

        assert!(reload <= 0x00ff_ffff);
        assert!(reload > 0);

        systick.disable_counter();
        systick.set_clock_source(cortex_m::peripheral::syst::SystClkSource::Core);
        systick.set_reload(reload);
        systick.enable_interrupt();
        systick.enable_counter();

        SYSTICK_TIMER_QUEUE.initialize(Systick {});
    }

    fn systick() -> SYST {
        unsafe { core::mem::transmute::<(), SYST>(()) }
    }
}

// Forward timerqueue interface
impl Systick {}

impl Monotonic for Systick {
    cfg_if::cfg_if! {
        if #[cfg(feature = "systick-64bit")] {
            type Instant = fugit::TimerInstantU64<TIMER_HZ>;
            type Duration = fugit::TimerDurationU64<TIMER_HZ>;
        } else {
            type Instant = fugit::TimerInstantU32<TIMER_HZ>;
            type Duration = fugit::TimerDurationU32<TIMER_HZ>;
        }
    }

    const ZERO: Self::Instant = Self::Instant::from_ticks(0);
    const TICK_PERIOD: Self::Duration = Self::Duration::from_ticks(1);

    fn now() -> Self::Instant {
        if Self::systick().has_wrapped() {
            SYSTICK_CNT.fetch_add(1, Ordering::AcqRel);
        }

        Self::Instant::from_ticks(SYSTICK_CNT.load(Ordering::Relaxed))
    }

    fn set_compare(_: Self::Instant) {
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

    fn enable_timer() {}

    fn disable_timer() {}

    fn __tq() -> &'static TimerQueue<Systick> {
        &SYSTICK_TIMER_QUEUE
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "systick-64bit")] {
        rtic_time::embedded_hal_delay_impl_fugit64!(Systick);

        #[cfg(feature = "embedded-hal-async")]
        rtic_time::embedded_hal_async_delay_impl_fugit64!(Systick);
    } else {
        rtic_time::embedded_hal_delay_impl_fugit32!(Systick);

        #[cfg(feature = "embedded-hal-async")]
        rtic_time::embedded_hal_async_delay_impl_fugit32!(Systick);
    }
}

/// Register the Systick interrupt for the monotonic.
#[macro_export]
macro_rules! create_systick_token {
    () => {{
        #[no_mangle]
        #[allow(non_snake_case)]
        unsafe extern "C" fn SysTick() {
            $crate::systick::Systick::__tq().on_monotonic_interrupt();
        }

        pub struct SystickToken;

        unsafe impl $crate::InterruptToken<$crate::systick::Systick> for SystickToken {}

        SystickToken
    }};
}
