//! ...

use super::Monotonic;
pub use super::{TimeoutError, TimerQueue};
use atomic_polyfill::{AtomicU32, Ordering};
use core::future::Future;
use cortex_m::peripheral::SYST;
use embedded_hal_async::delay::DelayUs;
pub use fugit::ExtU32;

// Features should be additive, here systick_100hz gets picked if both
// `systick_100hz` and `systick_10khz` are enabled.

cfg_if::cfg_if! {
    if #[cfg(feature = "systick_100hz")]
    {
        const TIMER_HZ: u32 = 100;
    } else if #[cfg(feature = "systick_10khz")]
    {
        const TIMER_HZ: u32 = 10_000;
    } else {
        // Default case is 1 kHz
        const TIMER_HZ: u32 = 1_000;
    }
}

/// Systick implementing `rtic_monotonic::Monotonic` which runs at 1 kHz, 100Hz or 10 kHz.
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
    pub fn start(mut systick: cortex_m::peripheral::SYST, sysclk: u32) {
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

static SYSTICK_CNT: AtomicU32 = AtomicU32::new(0);
static SYSTICK_TIMER_QUEUE: TimerQueue<Systick> = TimerQueue::new();

// Forward timerqueue interface
impl Systick {
    /// Used to access the underlying timer queue
    #[doc(hidden)]
    pub fn __tq() -> &'static TimerQueue<Systick> {
        &SYSTICK_TIMER_QUEUE
    }

    /// Timeout at a specific time.
    pub async fn timeout_at<F: Future>(
        instant: <Self as Monotonic>::Instant,
        future: F,
    ) -> Result<F::Output, TimeoutError> {
        SYSTICK_TIMER_QUEUE.timeout_at(instant, future).await
    }

    /// Timeout after a specific duration.
    #[inline]
    pub async fn timeout_after<F: Future>(
        duration: <Self as Monotonic>::Duration,
        future: F,
    ) -> Result<F::Output, TimeoutError> {
        SYSTICK_TIMER_QUEUE.timeout_after(duration, future).await
    }

    /// Delay for some duration of time.
    #[inline]
    pub async fn delay(duration: <Self as Monotonic>::Duration) {
        SYSTICK_TIMER_QUEUE.delay(duration).await;
    }

    /// Delay to some specific time instant.
    pub async fn delay_until(instant: <Self as Monotonic>::Instant) {
        SYSTICK_TIMER_QUEUE.delay_until(instant).await;
    }
}

impl Monotonic for Systick {
    type Instant = fugit::TimerInstantU32<TIMER_HZ>;
    type Duration = fugit::TimerDurationU32<TIMER_HZ>;

    const ZERO: Self::Instant = Self::Instant::from_ticks(0);

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
}

impl DelayUs for Systick {
    type Error = core::convert::Infallible;

    async fn delay_us(&mut self, us: u32) -> Result<(), Self::Error> {
        SYSTICK_TIMER_QUEUE.delay(us.micros()).await;
        Ok(())
    }

    async fn delay_ms(&mut self, ms: u32) -> Result<(), Self::Error> {
        SYSTICK_TIMER_QUEUE.delay(ms.millis()).await;
        Ok(())
    }
}

/// Register the Systick interrupt for the monotonic.
#[macro_export]
macro_rules! make_systick_handler {
    () => {
        #[no_mangle]
        #[allow(non_snake_case)]
        unsafe extern "C" fn SysTick() {
            Systick::__tq().on_monotonic_interrupt();
        }
    };
}
