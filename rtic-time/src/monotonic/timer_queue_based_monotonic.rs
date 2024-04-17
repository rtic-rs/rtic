use crate::{timer_queue::TimerQueueBackend, TimeoutError};

use crate::Monotonic;

/// A [`Monotonic`] that is backed by the [`TimerQueue`](crate::timer_queue::TimerQueue).
pub trait TimerQueueBasedMonotonic {
    /// The backend for the timer queue
    type Backend: TimerQueueBackend;

    /// The type for instant, defining an instant in time.
    ///
    /// **Note:** In all APIs in RTIC that use instants from this monotonic, this type will be used.
    type Instant: TimerQueueBasedInstant<Ticks = <Self::Backend as TimerQueueBackend>::Ticks>
        + core::ops::Add<Self::Duration, Output = Self::Instant>
        + core::ops::Sub<Self::Duration, Output = Self::Instant>
        + core::ops::Sub<Self::Instant, Output = Self::Duration>;

    /// The type for duration, defining a duration of time.
    ///
    /// **Note:** In all APIs in RTIC that use duration from this monotonic, this type will be used.
    type Duration: TimerQueueBasedDuration<Ticks = <Self::Backend as TimerQueueBackend>::Ticks>;
}

impl<T: TimerQueueBasedMonotonic> Monotonic for T {
    type Instant = T::Instant;
    type Duration = T::Duration;

    fn now() -> Self::Instant {
        Self::Instant::from_ticks(T::Backend::timer_queue().now())
    }

    async fn delay(duration: Self::Duration) {
        T::Backend::timer_queue().delay(duration.ticks()).await
    }

    async fn delay_until(instant: Self::Instant) {
        T::Backend::timer_queue().delay_until(instant.ticks()).await
    }

    async fn timeout_at<F: core::future::Future>(
        instant: Self::Instant,
        future: F,
    ) -> Result<F::Output, TimeoutError> {
        T::Backend::timer_queue()
            .timeout_at(instant.ticks(), future)
            .await
    }

    async fn timeout_after<F: core::future::Future>(
        duration: Self::Duration,
        future: F,
    ) -> Result<F::Output, TimeoutError> {
        T::Backend::timer_queue()
            .timeout_after(duration.ticks(), future)
            .await
    }
}

/// An instant that can be used in [`TimerQueueBasedMonotonic`].
pub trait TimerQueueBasedInstant: Ord + Copy {
    /// The internal type of the instant
    type Ticks;
    /// Convert from ticks to the instant
    fn from_ticks(ticks: Self::Ticks) -> Self;
    /// Convert the instant to ticks
    fn ticks(self) -> Self::Ticks;
}

/// A duration that can be used in [`TimerQueueBasedMonotonic`].
pub trait TimerQueueBasedDuration: Copy {
    /// The internal type of the duration
    type Ticks;
    /// Convert the duration to ticks
    fn ticks(self) -> Self::Ticks;
}

impl<const NOM: u32, const DENOM: u32> TimerQueueBasedInstant for fugit::Instant<u64, NOM, DENOM> {
    type Ticks = u64;
    fn from_ticks(ticks: Self::Ticks) -> Self {
        Self::from_ticks(ticks)
    }
    fn ticks(self) -> Self::Ticks {
        Self::ticks(&self)
    }
}

impl<const NOM: u32, const DENOM: u32> TimerQueueBasedInstant for fugit::Instant<u32, NOM, DENOM> {
    type Ticks = u32;
    fn from_ticks(ticks: Self::Ticks) -> Self {
        Self::from_ticks(ticks)
    }
    fn ticks(self) -> Self::Ticks {
        Self::ticks(&self)
    }
}

impl<const NOM: u32, const DENOM: u32> TimerQueueBasedDuration
    for fugit::Duration<u64, NOM, DENOM>
{
    type Ticks = u64;
    fn ticks(self) -> Self::Ticks {
        Self::ticks(&self)
    }
}

impl<const NOM: u32, const DENOM: u32> TimerQueueBasedDuration
    for fugit::Duration<u32, NOM, DENOM>
{
    type Ticks = u32;
    fn ticks(self) -> Self::Ticks {
        Self::ticks(&self)
    }
}
