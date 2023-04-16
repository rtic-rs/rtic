//! A monotonic clock / counter definition.

/// # A monotonic clock / counter definition.
///
/// ## Correctness
///
/// The trait enforces that proper time-math is implemented between `Instant` and `Duration`. This
/// is a requirement on the time library that the user chooses to use.
pub trait Monotonic {
    /// The time at time zero.
    const ZERO: Self::Instant;

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
}
