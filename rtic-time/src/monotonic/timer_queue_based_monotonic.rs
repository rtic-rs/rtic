use crate::{timer_queue::TimerQueueBackend, TimeoutError};

use super::Monotonic;

/// A monotonic that is backed by the [timer queue](crate::timer_queue::TimerQueue).
pub trait TimerQueueBasedMonotonic {
    /// The backend for the timer queue
    type Backend: TimerQueueBackend;

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
