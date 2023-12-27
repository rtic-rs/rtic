//! A monotonic clock / counter definition.

use core::marker::PhantomData;

use crate::TimeoutError;

/// # A monotonic clock / counter definition.
///
/// ## Correctness
///
/// The trait enforces that proper time-math is implemented between `Instant` and `Duration`. This
/// is a requirement on the time library that the user chooses to use.
pub trait Monotonic {
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

    /// The type for duration, defining an duration of time.
    ///
    /// **Note:** In all APIs in RTIC that use duration from this monotonic, this type will be used.
    type Duration: Copy;

    /// Get the current time.
    fn now() -> Self::Instant;

    /// Delay for some duration of time.
    async fn delay(duration: Self::Duration);

    /// Delay to some specific time instant.
    async fn delay_until(instant: Self::Instant);

    /// Timeout at a specific time.
    async fn timeout_at<F: core::future::Future>(
        instant: Self::Instant,
        future: F,
    ) -> Result<F::Output, TimeoutError>;

    /// Timeout after a specific duration.
    async fn timeout_after<F: core::future::Future>(
        duration: Self::Duration,
        future: F,
    ) -> Result<F::Output, TimeoutError>;

    /// A duration which represents the specified amount of milliseconds,
    /// rounded up.
    ///
    /// Required for embedded-hal implementations.
    fn duration_millis_at_least(ms: u32) -> Self::Duration;

    /// A duration which represents the specified amount of microseconds,
    /// rounded up.
    ///
    /// Required for embedded-hal implementations.
    fn duration_micros_at_least(us: u32) -> Self::Duration;

    /// A duration which represents the specified amount of nanoseconds,
    /// rounded up.
    ///
    /// Required for embedded-hal implementations.
    fn duration_nanos_at_least(ns: u32) -> Self::Duration;
}

struct MonotonicWrapper<Mono: Monotonic> {
    _m: PhantomData<Mono>,
}

impl<T: Monotonic> Monotonic for MonotonicWrapper<T> {
    const ZERO: T::Instant = T::ZERO;

    const TICK_PERIOD: T::Duration = T::TICK_PERIOD;

    type Instant = T::Instant;

    type Duration = T::Duration;

    fn now() -> Self::Instant {
        T::now()
    }

    async fn delay(duration: Self::Duration) {
        T::delay(duration).await
    }

    async fn delay_until(instant: Self::Instant) {
        T::delay_until(instant).await
    }

    async fn timeout_at<F: core::future::Future>(
        instant: Self::Instant,
        future: F,
    ) -> Result<F::Output, TimeoutError> {
        T::timeout_at(instant, future).await
    }

    async fn timeout_after<F: core::future::Future>(
        duration: Self::Duration,
        future: F,
    ) -> Result<F::Output, TimeoutError> {
        T::timeout_after(duration, future).await
    }

    fn duration_millis_at_least(ms: u32) -> Self::Duration {
        T::duration_millis_at_least(ms)
    }

    fn duration_micros_at_least(us: u32) -> Self::Duration {
        T::duration_micros_at_least(us)
    }

    fn duration_nanos_at_least(ns: u32) -> Self::Duration {
        T::duration_nanos_at_least(ns)
    }
}

impl<T: Monotonic> embedded_hal::delay::DelayNs for MonotonicWrapper<T> {
    fn delay_ns(&mut self, ns: u32) {
        let now = Self::now();
        let mut done = now + Self::duration_nanos_at_least(ns);
        if now != done {
            // Compensate for sub-tick uncertainty
            done = done + Self::TICK_PERIOD;
        }

        while Self::now() < done {}
    }

    fn delay_us(&mut self, us: u32) {
        let now = Self::now();
        let mut done = now + Self::duration_micros_at_least(us);
        if now != done {
            // Compensate for sub-tick uncertainty
            done = done + Self::TICK_PERIOD;
        }

        while Self::now() < done {}
    }

    fn delay_ms(&mut self, ms: u32) {
        let now = Self::now();
        let mut done = now + Self::duration_millis_at_least(ms);
        if now != done {
            // Compensate for sub-tick uncertainty
            done = done + Self::TICK_PERIOD;
        }

        while Self::now() < done {}
    }
}

impl<T: Monotonic> embedded_hal_async::delay::DelayNs for MonotonicWrapper<T> {
    #[inline]
    async fn delay_ns(&mut self, ns: u32) {
        Self::delay(Self::duration_nanos_at_least(ns)).await;
    }

    #[inline]
    async fn delay_us(&mut self, us: u32) {
        Self::delay(Self::duration_micros_at_least(us)).await;
    }

    #[inline]
    async fn delay_ms(&mut self, ms: u32) {
        Self::delay(Self::duration_millis_at_least(ms)).await;
    }
}
