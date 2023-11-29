//! A monotonic clock / counter definition.

use crate::{TimeoutError, TimerQueue};

/// # A monotonic clock / counter definition.
///
/// ## Correctness
///
/// The trait enforces that proper time-math is implemented between `Instant` and `Duration`. This
/// is a requirement on the time library that the user chooses to use.
pub trait Monotonic: Sized + 'static {
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
    type Duration;

    /// Get the current time.
    fn now() -> Self::Instant;

    /// Set the compare value of the timer interrupt.
    ///
    /// **Note:** This method does not need to handle race conditions of the monotonic, the timer
    /// queue in RTIC checks this.
    fn set_compare(instant: Self::Instant);

    /// Override for the dequeue check, override with timers that have bugs.
    ///
    /// E.g. nRF52 RTCs needs to be dequeued if the time is within 4 ticks.
    fn should_dequeue_check(release_at: Self::Instant) -> bool {
        <Self as Monotonic>::now() >= release_at
    }

    /// Clear the compare interrupt flag.
    fn clear_compare_flag();

    /// Pend the timer's interrupt.
    fn pend_interrupt();

    /// Optional. Runs on interrupt before any timer queue handling.
    fn on_interrupt() {}

    /// Optional. This is used to save power, this is called when the timer queue is not empty.
    ///
    /// Enabling and disabling the monotonic needs to propagate to `now` so that an instant
    /// based of `now()` is still valid.
    ///
    /// NOTE: This may be called more than once.
    fn enable_timer() {}

    /// Optional. This is used to save power, this is called when the timer queue is empty.
    ///
    /// Enabling and disabling the monotonic needs to propagate to `now` so that an instant
    /// based of `now()` is still valid.
    ///
    /// NOTE: This may be called more than once.
    fn disable_timer() {}

    /// Return a reference to the underlying timer queue
    #[doc(hidden)]
    fn __tq() -> &'static TimerQueue<Self>;

    /// Delay for some duration of time.
    #[inline]
    fn delay(duration: <Self as Monotonic>::Duration) -> impl core::future::Future<Output = ()> {
        async move {
            Self::__tq().delay(duration).await;
        }
    }

    /// Timeout at a specific time.
    fn timeout_at<F: core::future::Future>(
        instant: Self::Instant,
        future: F,
    ) -> impl core::future::Future<Output = Result<F::Output, TimeoutError>> {
        async move { Self::__tq().timeout_at(instant, future).await }
    }

    /// Timeout after a specific duration.
    #[inline]
    fn timeout_after<F: core::future::Future>(
        duration: <Self as Monotonic>::Duration,
        future: F,
    ) -> impl core::future::Future<Output = Result<F::Output, TimeoutError>> {
        async move { Self::__tq().timeout_after(duration, future).await }
    }

    /// Delay to some specific time instant.
    #[inline]
    fn delay_until(
        instant: <Self as Monotonic>::Instant,
    ) -> impl core::future::Future<Output = ()> {
        async move {
            TimerQueue::<Self>::new();
            Self::__tq().delay_until(instant).await;
        }
    }
}

/// Creates impl blocks for `embedded_hal::delay::DelayUs` and
/// `embedded_hal_async::delay::DelayUs`, based on `fugit::ExtU64Ceil`.
#[macro_export]
macro_rules! embedded_hal_delay_impl_fugit64 {
    ($t:ty) => {
        #[cfg(feature = "embedded-hal-async")]
        impl ::embedded_hal_async::delay::DelayUs for $t {
            #[inline]
            async fn delay_us(&mut self, us: u32) {
                use ::fugit::ExtU64Ceil;
                Self::delay(u64::from(us).micros_at_least()).await;
            }

            #[inline]
            async fn delay_ms(&mut self, ms: u32) {
                use ::fugit::ExtU64Ceil;
                Self::delay(u64::from(ms).millis_at_least()).await;
            }
        }

        impl ::embedded_hal::delay::DelayUs for $t {
            fn delay_us(&mut self, us: u32) {
                use ::fugit::ExtU64Ceil;
                let done = Self::now() + u64::from(us).micros_at_least() + Self::TICK_PERIOD;
                while Self::now() < done {}
            }
            fn delay_ms(&mut self, ms: u32) {
                use ::fugit::ExtU64Ceil;
                let done = Self::now() + u64::from(ms).millis_at_least() + Self::TICK_PERIOD;
                while Self::now() < done {}
            }
        }
    };
}

/// Creates impl blocks for `embedded_hal::delay::DelayUs` and
/// `embedded_hal_async::delay::DelayUs`, based on `fugit::ExtU32Ceil`.
#[macro_export]
macro_rules! embedded_hal_delay_impl_fugit32 {
    ($t:ty) => {
        #[cfg(feature = "embedded-hal-async")]
        impl ::embedded_hal_async::delay::DelayUs for $t {
            #[inline]
            async fn delay_us(&mut self, us: u32) {
                use ::fugit::ExtU32Ceil;
                Self::delay(us.micros_at_least()).await;
            }

            #[inline]
            async fn delay_ms(&mut self, ms: u32) {
                use ::fugit::ExtU32Ceil;
                Self::delay(ms.millis_at_least()).await;
            }
        }

        impl ::embedded_hal::delay::DelayUs for $t {
            fn delay_us(&mut self, us: u32) {
                use ::fugit::ExtU32Ceil;
                let done = Self::now() + us.micros_at_least() + Self::TICK_PERIOD;
                while Self::now() < done {}
            }
            fn delay_ms(&mut self, ms: u32) {
                use ::fugit::ExtU32Ceil;
                let done = Self::now() + ms.millis_at_least() + Self::TICK_PERIOD;
                while Self::now() < done {}
            }
        }
    };
}
