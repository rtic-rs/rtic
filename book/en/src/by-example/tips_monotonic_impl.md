# Implementing a `Monotonic` timer for scheduling

The framework is very flexible in that it can utilize any timer which has compare-match and (optional)
overflow interrupts for scheduling. The only thing needed to make a timer usable with RTIC is to
implement the [`rtic_monotonic::Monotonic`] trait.

Implementing time that supports a vast range is generally **very** difficult, and in RTIC 0.5 it was a
common problem how to implement time handling and not get stuck in weird special cases. Moreover
it was difficult to understand the relation between time and the timers used for scheduling. From
RTIC 0.6 we have moved to use [`embedded_time`] as the basis for all time-based operation and
abstraction of clocks. This is why from RTIC 0.6 it is almost trivial to implement the `Monotonic`
trait and use any timer in a system for scheduling.

The trait documents the requirements for each method, however a small PoC implementation is provided
below.

[`rtic_monotonic::Monotonic`]: https://docs.rs/rtic-monotonic/
[`embedded_time`]: https://docs.rs/embedded_time/

```rust
use rtic_monotonic::{embedded_time::clock::Error, Clock, Fraction, Instant, Monotonic};

/// Example wrapper struct for a timer
pub struct Timer<const FREQ: u32> {
    tim: TIM2,
}

impl<const FREQ: u32> Clock for Timer<FREQ> {
    const SCALING_FACTOR: Fraction = Fraction::new(1, FREQ);
    type T = u32;

    #[inline(always)]
    fn try_now(&self) -> Result<Instant<Self>, Error> {
        Ok(Instant::new(Self::count()))
    }
}

impl Monotonic for Timer<TIM2> {
    unsafe fn reset(&mut self) {
        // Reset timer counter
        self.tim.cnt.write(|_, w| w.bits(0));

        // Since reset is only called once, we use it to enable
        // the interrupt generation bit.
        self.tim.dier.modify(|_, w| w.cc1ie().set_bit());
    }

    // Use Compare channel 1 for Monotonic
    fn set_compare(&mut self, instant: &Instant<Self>) {
        self.tim
            .ccr1
            .write(|w| w.ccr().bits(instant.duration_since_epoch().integer()));
    }

    fn clear_compare_flag(&mut self) {
        self.tim.sr.modify(|_, w| w.cc1if().clear_bit());
    }
}
```
