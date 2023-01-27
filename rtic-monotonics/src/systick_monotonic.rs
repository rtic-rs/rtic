//! ...

use super::Monotonic;
pub use super::{TimeoutError, TimerQueue};
use atomic_polyfill::{AtomicU32, Ordering};
use core::ops::Deref;
use cortex_m::peripheral::SYST;
use embedded_hal_async::delay::DelayUs;
pub use fugit::ExtU32;

const TIMER_HZ: u32 = 1_000;

/// Systick implementing `rtic_monotonic::Monotonic` which runs at 1 kHz.
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
    #[must_use]
    pub fn start(mut systick: cortex_m::peripheral::SYST, sysclk: u32) -> Self {
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

        Systick {}
    }

    fn systick() -> SYST {
        unsafe { core::mem::transmute::<(), SYST>(()) }
    }
}

static SYSTICK_CNT: AtomicU32 = AtomicU32::new(0);

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

/// Timer queue wrapper to implement traits on
pub struct SystickTimerQueue(TimerQueue<Systick>);

impl SystickTimerQueue {
    /// Create a new timer queue.
    pub const fn new() -> Self {
        Self(TimerQueue::new())
    }
}

impl Deref for SystickTimerQueue {
    type Target = TimerQueue<Systick>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DelayUs for SystickTimerQueue {
    type Error = core::convert::Infallible;

    async fn delay_us(&mut self, us: u32) -> Result<(), Self::Error> {
        self.delay(us.micros()).await;
        Ok(())
    }

    async fn delay_ms(&mut self, ms: u32) -> Result<(), Self::Error> {
        self.delay(ms.millis()).await;
        Ok(())
    }
}

/// Register the Systick interrupt and crate a timer queue with a specific name and speed.
#[macro_export]
macro_rules! make_systick_timer_queue {
    ($timer_queue_name:ident) => {
        static $timer_queue_name: SystickTimerQueue = SystickTimerQueue::new();

        #[no_mangle]
        #[allow(non_snake_case)]
        unsafe extern "C" fn SysTick() {
            $timer_queue_name.on_monotonic_interrupt();
        }
    };
}
