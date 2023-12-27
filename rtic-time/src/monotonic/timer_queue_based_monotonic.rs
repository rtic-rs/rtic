use crate::{timer_queue::TimerQueueBackend, TimeoutError};

use super::Monotonic;

/// A monotonic that is backed by the [timer queue](crate::timer_queue::TimerQueue).
pub trait TimerQueueBasedMonotonic {
    /// The backend for the timer queue
    type Backend: TimerQueueBackend;

    /// The time at time zero.
    const ZERO: Self::Instant;

    /// The duration between two timer ticks.
    const TICK_PERIOD: Self::Duration;

    /// The type for instant, defining an instant in time.
    ///
    /// **Note:** In all APIs in RTIC that use instants from this monotonic, this type will be used.
    type Instant: Ord
        + Copy
        + core::ops::Add<Self::Duration, Output = Self::Instant>
        + core::ops::Sub<Self::Duration, Output = Self::Instant>
        + core::ops::Sub<Self::Instant, Output = Self::Duration>;

    /// The type for duration, defining a duration of time.
    ///
    /// **Note:** In all APIs in RTIC that use duration from this monotonic, this type will be used.
    type Duration: Copy;

    /// Converts ticks to `Instant`.
    fn ticks_to_instant(ticks: <Self::Backend as TimerQueueBackend>::Ticks) -> Self::Instant;
    /// Converts `Instant` to ticks.
    fn instant_to_ticks(instant: Self::Instant) -> <Self::Backend as TimerQueueBackend>::Ticks;
    /// Converts `Duration` to ticks.
    fn duration_to_ticks(duration: Self::Duration) -> <Self::Backend as TimerQueueBackend>::Ticks;
}

impl<T: TimerQueueBasedMonotonic> Monotonic for T {
    const ZERO: T::Instant = T::ZERO;

    const TICK_PERIOD: T::Duration = T::TICK_PERIOD;

    type Instant = T::Instant;

    type Duration = T::Duration;

    fn now() -> Self::Instant {
        Self::ticks_to_instant(T::Backend::timer_queue().now())
    }

    async fn delay(duration: Self::Duration) {
        T::Backend::timer_queue()
            .delay(Self::duration_to_ticks(duration))
            .await
    }

    async fn delay_until(instant: Self::Instant) {
        T::Backend::timer_queue()
            .delay_until(Self::instant_to_ticks(instant))
            .await
    }

    async fn timeout_at<F: core::future::Future>(
        instant: Self::Instant,
        future: F,
    ) -> Result<F::Output, TimeoutError> {
        T::Backend::timer_queue()
            .timeout_at(Self::instant_to_ticks(instant), future)
            .await
    }

    async fn timeout_after<F: core::future::Future>(
        duration: Self::Duration,
        future: F,
    ) -> Result<F::Output, TimeoutError> {
        T::Backend::timer_queue()
            .timeout_after(Self::duration_to_ticks(duration), future)
            .await
    }
}

// impl<T: TimerQueueBasedMonotonic> embedded_hal::delay::DelayNs for T {
//     fn delay_ns(&mut self, ns: u32) {
//         let now = Self::now();
//         let mut done = now + Self::duration_nanos_at_least(ns);
//         if now != done {
//             // Compensate for sub-tick uncertainty
//             done = done + Self::TICK_PERIOD;
//         }

//         while Self::now() < done {}
//     }

//     fn delay_us(&mut self, us: u32) {
//         let now = Self::now();
//         let mut done = now + Self::duration_micros_at_least(us);
//         if now != done {
//             // Compensate for sub-tick uncertainty
//             done = done + Self::TICK_PERIOD;
//         }

//         while Self::now() < done {}
//     }

//     fn delay_ms(&mut self, ms: u32) {
//         let now = Self::now();
//         let mut done = now + Self::duration_millis_at_least(ms);
//         if now != done {
//             // Compensate for sub-tick uncertainty
//             done = done + Self::TICK_PERIOD;
//         }

//         while Self::now() < done {}
//     }
// }

// impl<T: TimerQueueBasedMonotonic> embedded_hal_async::delay::DelayNs for T {
//     #[inline]
//     async fn delay_ns(&mut self, ns: u32) {
//         Self::delay(Self::duration_nanos_at_least(ns)).await;
//     }

//     #[inline]
//     async fn delay_us(&mut self, us: u32) {
//         Self::delay(Self::duration_micros_at_least(us)).await;
//     }

//     #[inline]
//     async fn delay_ms(&mut self, ms: u32) {
//         Self::delay(Self::duration_millis_at_least(ms)).await;
//     }
// }
