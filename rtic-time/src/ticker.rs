//! A periodic timer that does not drift.

use crate::Timebase;

/// Yields ticks at a fixed period.
///
/// Tick `n` is scheduled at `start + n * period`, so wake-up latency does not add up over time.
///
/// If ticks are missed, the following calls to [`tick`](Ticker::tick) complete at once until the
/// schedule is caught up. Use [`reset`](Ticker::reset) to drop the missed ticks instead.
///
/// On a wrapping timebase the ticker must not fall behind by more than half the counter range,
/// as a missed tick then looks like a tick in the future.
///
/// ```ignore
/// let mut ticker = Mono::ticker(1.secs());
/// loop {
///     let at = ticker.tick().await;
/// }
/// ```
pub struct Ticker<T: Timebase> {
    next: T::Instant,
    period: T::Duration,
}

impl<T: Timebase> Ticker<T> {
    /// Create a ticker with its first tick one `period` from now.
    pub fn new(period: T::Duration) -> Self {
        Self::starting_at(T::now() + period, period)
    }

    /// Create a ticker with its first tick at `start`.
    pub fn starting_at(start: T::Instant, period: T::Duration) -> Self {
        Self {
            next: start,
            period,
        }
    }

    /// Wait for the next tick and return the instant it was scheduled for.
    ///
    /// This is cancel safe. If the future is dropped before it completes, no tick is lost.
    pub async fn tick(&mut self) -> T::Instant {
        T::delay_until(self.next).await;
        let at = self.next;
        self.next = at + self.period;
        at
    }

    /// Move the schedule so the next tick is one period from now.
    pub fn reset(&mut self) {
        self.next = T::now() + self.period;
    }

    /// The period between ticks.
    pub fn period(&self) -> T::Duration {
        self.period
    }
}
