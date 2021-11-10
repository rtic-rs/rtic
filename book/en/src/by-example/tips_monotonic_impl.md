# Implementing a `Monotonic` timer for scheduling

The framework is very flexible in that it can utilize any timer which has compare-match and (optional)
overflow interrupts for scheduling. The only thing needed to make a timer usable with RTIC is to
implement the [`rtic_monotonic::Monotonic`] trait.

Implementing time that supports a vast range is generally **very** difficult, and in RTIC 0.5 it was a
common problem how to implement time handling and not get stuck in weird special cases. Moreover
it was difficult to understand the relation between time and the timers used for scheduling. For
RTIC 0.6 we have moved to assume the user has a time library, e.g. [`fugit`] or [`embedded_time`],
as the basis for all time-based operations when implementing `Monotonic`. This is why in RTIC 0.6
it is almost trivial to implement the `Monotonic` trait and use any timer in a system for scheduling.

The trait documents the requirements for each method, however a small PoC implementation is provided
below.

[`rtic_monotonic::Monotonic`]: https://docs.rs/rtic-monotonic/
[`fugit`]: https://docs.rs/fugit/
[`embedded_time`]: https://docs.rs/embedded_time/

```rust
pub use fugit::{self, ExtU32};
use rtic_monotonic::Monotonic;

/// Example wrapper struct for a timer
pub struct Timer<const TIMER_HZ: u32> {
    tim: TIM2,
}

impl<const TIMER_HZ: u32> Monotonic for Timer<TIMER_HZ> {
    type Instant = fugit::TimerInstantU32<TIMER_HZ>;
    type Duration = fugit::TimerDurationU32<TIMER_HZ>;

    fn now(&mut self) -> Self::Instant {
        // Read the timer count
        Self::Instant::from_ticks(Self::count())
    }

    fn zero() -> Self::Instant {
        // This is used while the app is in `#[init]`, if the system cannot
        // support time in `#[init]` this can also be a `panic!(..)`
        Self::Instant::from_ticks(0)
    }

    unsafe fn reset(&mut self) {
        // Reset timer counter
        self.tim.cnt.write(|_, w| w.bits(0));

        // Since reset is only called once, we use it to enable
        // the interrupt generation bit.
        self.tim.dier.modify(|_, w| w.cc1ie().set_bit());
    }

    fn set_compare(&mut self, instant: Instant<Self>) {
        // Use Compare channel 1 for Monotonic
        self.tim
            .ccr1
            .write(|w| w.ccr().bits(instant.ticks()));
    }

    fn clear_compare_flag(&mut self) {
        self.tim.sr.modify(|_, w| w.cc1if().clear_bit());
    }
}
```
