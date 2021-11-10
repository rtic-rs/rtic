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

The trait documents the requirements for each method, however below you can find a list of
implementations in the wild that can be used as inspiration:

- [`STM32F411 series`], implemented for the 32-bit timers
- [`Nordic nRF52 series`], implemented for the 32-bit timers
- [`Systick based`], runs at a fixed rate - some overhead but simple
- [`DWT and Systick based`], a more efficient `Systick` based implementation, but requires `DWT`

If you know of more implementations feel free to add them to this list.

[`rtic_monotonic::Monotonic`]: https://docs.rs/rtic-monotonic/
[`fugit`]: https://docs.rs/fugit/
[`embedded_time`]: https://docs.rs/embedded_time/
[`STM32F411 series`]: https://github.com/kalkyl/f411-rtic/blob/main/src/bin/mono.rs
[`Nordic nRF52 series`]: https://github.com/kalkyl/nrf-play/blob/main/src/bin/mono.rs
[`Systick based`]: https://github.com/rtic-rs/systick-monotonic
[`DWT and Systick based`]: https://github.com/rtic-rs/dwt-systick-monotonic

