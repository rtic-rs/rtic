use super::{TimerQueue, TimerQueueTicks};

/// A backend definition for a monotonic clock/counter.
pub trait TimerQueueBackend: 'static + Sized {
    /// The type for ticks.
    type Ticks: TimerQueueTicks;

    /// Get the current time.
    fn now() -> Self::Ticks;

    /// Set the compare value of the timer interrupt.
    ///
    /// **Note:** This method does not need to handle race conditions of the monotonic, the timer
    /// queue in RTIC checks this.
    fn set_compare(instant: Self::Ticks);

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

    /// Returns a reference to the underlying timer queue.
    fn timer_queue() -> &'static TimerQueue<Self>;
}
