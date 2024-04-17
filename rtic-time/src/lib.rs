//! Time-related traits & structs.
//!
//! This crate contains basic definitions and utilities that can be used
//! to keep track of time.

#![no_std]
#![deny(missing_docs)]
#![allow(async_fn_in_trait)]

pub mod half_period_counter;
mod linked_list;
pub mod monotonic;
pub mod timer_queue;

/// This indicates that there was a timeout.
pub struct TimeoutError;

/// Re-export for macros
pub use embedded_hal;
/// Re-export for macros
pub use embedded_hal_async;

/// # A monotonic clock / counter definition.
///
/// ## Correctness
///
/// The trait enforces that proper time-math is implemented between `Instant` and `Duration`. This
/// is a requirement on the time library that the user chooses to use.
pub trait Monotonic {
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
}
