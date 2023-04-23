//! [`Monotonic`] implementation for RP2040's Timer peripheral.
//!
//! # Example
//!
//! ```
//! use rtic_monotonics::rp2040::*;
//!
//! fn init() {
//!     # // This is normally provided by the selected PAC
//!     # let timer = unsafe { core::mem::transmute(()) };
//!     # let mut resets = unsafe { core::mem::transmute(()) };
//!     // Generate the required token
//!     let token = rtic_monotonics::create_rp2040_monotonic_token!();
//!
//!     // Start the monotonic
//!     Timer::start(timer, &mut resets, token);
//! }
//!
//! async fn usage() {
//!     loop {
//!          // Use the monotonic
//!          Timer::delay(100.millis()).await;
//!     }
//! }
//! ```

use super::Monotonic;

pub use super::{TimeoutError, TimerQueue};
use core::future::Future;
pub use fugit::{self, ExtU64};
use rp2040_pac::{timer, Interrupt, NVIC, RESETS, TIMER};

/// Timer implementing `rtic_monotonic::Monotonic` which runs at 1 MHz.
pub struct Timer;

impl Timer {
    /// Start a `Monotonic` based on RP2040's Timer.
    pub fn start(
        timer: TIMER,
        resets: &mut RESETS,
        _interrupt_token: impl crate::InterruptToken<Self>,
    ) {
        resets.reset.modify(|_, w| w.timer().clear_bit());
        while resets.reset_done.read().timer().bit_is_clear() {}
        timer.inte.modify(|_, w| w.alarm_0().set_bit());

        TIMER_QUEUE.initialize(Self {});

        unsafe {
            crate::set_monotonic_prio(rp2040_pac::NVIC_PRIO_BITS, Interrupt::TIMER_IRQ_0);
            NVIC::unmask(Interrupt::TIMER_IRQ_0);
        }
    }

    fn timer() -> &'static timer::RegisterBlock {
        unsafe { &*TIMER::ptr() }
    }
}

static TIMER_QUEUE: TimerQueue<Timer> = TimerQueue::new();

// Forward timerqueue interface
impl Timer {
    /// Used to access the underlying timer queue
    #[doc(hidden)]
    pub fn __tq() -> &'static TimerQueue<Timer> {
        &TIMER_QUEUE
    }

    /// Timeout at a specific time.
    #[inline]
    pub async fn timeout_at<F: Future>(
        instant: <Self as Monotonic>::Instant,
        future: F,
    ) -> Result<F::Output, TimeoutError> {
        TIMER_QUEUE.timeout_at(instant, future).await
    }

    /// Timeout after a specific duration.
    #[inline]
    pub async fn timeout_after<F: Future>(
        duration: <Self as Monotonic>::Duration,
        future: F,
    ) -> Result<F::Output, TimeoutError> {
        TIMER_QUEUE.timeout_after(duration, future).await
    }

    /// Delay for some duration of time.
    #[inline]
    pub async fn delay(duration: <Self as Monotonic>::Duration) {
        TIMER_QUEUE.delay(duration).await;
    }

    /// Delay to some specific time instant.
    #[inline]
    pub async fn delay_until(instant: <Self as Monotonic>::Instant) {
        TIMER_QUEUE.delay_until(instant).await;
    }
}

impl Monotonic for Timer {
    type Instant = fugit::TimerInstantU64<1_000_000>;
    type Duration = fugit::TimerDurationU64<1_000_000>;

    const ZERO: Self::Instant = Self::Instant::from_ticks(0);

    fn now() -> Self::Instant {
        let timer = Self::timer();

        let mut hi0 = timer.timerawh.read().bits();
        loop {
            let low = timer.timerawl.read().bits();
            let hi1 = timer.timerawh.read().bits();
            if hi0 == hi1 {
                break Self::Instant::from_ticks((u64::from(hi0) << 32) | u64::from(low));
            }
            hi0 = hi1;
        }
    }

    fn set_compare(instant: Self::Instant) {
        let now = Self::now();

        let max = u32::MAX as u64;

        // Since the timer may or may not overflow based on the requested compare val, we check
        // how many ticks are left.
        let val = match instant.checked_duration_since(now) {
            Some(x) if x.ticks() <= max => instant.duration_since_epoch().ticks() & max, // Will not overflow
            _ => 0, // Will overflow or in the past, set the same value as after overflow to not get extra interrupts
        };

        Self::timer()
            .alarm0
            .write(|w| unsafe { w.bits(val as u32) });
    }

    fn clear_compare_flag() {
        Self::timer().intr.modify(|_, w| w.alarm_0().set_bit());
    }

    fn pend_interrupt() {
        rp2040_pac::NVIC::pend(Interrupt::TIMER_IRQ_0);
    }

    fn on_interrupt() {}

    fn enable_timer() {}

    fn disable_timer() {}
}

#[cfg(feature = "embedded-hal-async")]
impl embedded_hal_async::delay::DelayUs for Timer {
    async fn delay_us(&mut self, us: u32) {
        Self::delay((us as u64).micros()).await;
    }

    async fn delay_ms(&mut self, ms: u32) {
        Self::delay((ms as u64).millis()).await;
    }
}

/// Register the Timer interrupt for the monotonic.
#[macro_export]
macro_rules! create_rp2040_monotonic_token {
    () => {{
        #[no_mangle]
        #[allow(non_snake_case)]
        unsafe extern "C" fn TIMER_IRQ_0() {
            $crate::rp2040::Timer::__tq().on_monotonic_interrupt();
        }

        pub struct Rp2040Token;

        unsafe impl $crate::InterruptToken<$crate::rp2040::Timer> for Rp2040Token {}

        Rp2040Token
    }};
}
