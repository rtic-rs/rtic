# The magic behind Monotonics

Internally, all monotonics use a [Timer Queue](#the-timer-queue), which is a priority queue with entries describing the time at which their respective `Future`s should complete.

## Implementing a `Monotonic` timer for scheduling

The [`rtic-time`] framework is flexible because it can use any timer which has compare-match and optionally supporting overflow interrupts for scheduling. The single requirement to make a timer usable with RTIC is implementing the [`rtic-time::Monotonic`] trait.

For RTIC 2.0, we assume that the user has a time library, e.g. [`fugit`], as the basis for all time-based operations when implementing [`Monotonic`]. These libraries make it much easier to correctly implement the [`Monotonic`] trait, allowing the use of almost any timer in the system for scheduling.

The trait documents the requirements for each method. There are reference implementations available in [`rtic-monotonics`] that can be used for inspiration.

- [`Systick based`], runs at a fixed interrupt (core tick) rate - simple, but comes with some overhead and can only be used while the core is running
- [`RP2040 Timer`], a "proper" implementation with support for waiting for long periods without interrupts. Clearly demonstrates how to use the [`TimerQueue`] to handle scheduling.
- [`nRF52 timers`] implements monotonic & Timer Queue for the RTC and normal timers in nRF52's

Often, hardware counters only have a width of only 16 or 32 bit, meaning that they would overflow after some minutes to years, depending on the frequency of the counter peripheral. To overcome this issue, monotonic implementations for such counters must use an atomic overflow counter (e.g. [`portable_atomic::AtomicU32`]) to correctly calculate the current time instant. This leads to a race condition between the overflow counter and the time register, which isn't trivial to solve. Hence, a half period counter must be used. Please read and refer to [`rtic_time::half_period_counter`], which provides helper methods and examples for implementing this logic.

## Contributing

Contributing new implementations of `Monotonic` can be done in multiple ways:
* Implement the trait behind a feature flag in [`rtic-monotonics`], and create a PR for them to be included in the main RTIC repository. This way, the implementations of are in-tree, RTIC can guarantee their correctness, and can update them in the case of a new release.
* Implement the changes in an external repository. Doing so will not have them included in [`rtic-monotonics`], but may make it easier to do so in the future.

[`rtic-monotonics`]: https://github.com/rtic-rs/rtic/tree/master/rtic-monotonics/
[`fugit`]: https://docs.rs/fugit/
[`Systick based`]: https://github.com/rtic-rs/rtic/blob/master/rtic-monotonics/src/systick.rs
[`rtic-monotonics`]:  https://github.com/rtic-rs/rtic/blob/master/rtic-monotonics
[`RP2040 Timer`]: https://github.com/rtic-rs/rtic/blob/master/rtic-monotonics/src/rp2040.rs
[`nRF52 timers`]: https://github.com/rtic-rs/rtic/blob/master/rtic-monotonics/src/nrf.rs
[`rtic-time`]: https://docs.rs/rtic-time/latest/rtic_time
[`rtic-time::Monotonic`]: https://docs.rs/rtic-time/latest/rtic_time/trait.Monotonic.html
[`Monotonic`]: https://docs.rs/rtic-time/latest/rtic_time/trait.Monotonic.html
[`TimerQueue`]: https://docs.rs/rtic-time/latest/rtic_time/timer_queue/struct.TimerQueue.html
[`rtic_time::half_period_counter`]: https://docs.rs/rtic-time/latest/rtic_time/half_period_counter/
[`portable_atomic::AtomicU32`]: https://docs.rs/portable-atomic/latest/portable_atomic/struct.AtomicU32.html

## The timer queue

The timer queue is implemented as a list based priority queue, where list-nodes are statically allocated as part of the `Future` created when `await`-ing a Future created when waiting for the monotonic. Thus, the timer queue is infallible at run-time (its size and allocation are determined at compile time).

Similarly the channels implementation, the timer-queue implementation relies on a global *Critical Section* (CS) for race protection. For the examples a CS implementation is provided by adding `--features test-critical-section` to the build options.
