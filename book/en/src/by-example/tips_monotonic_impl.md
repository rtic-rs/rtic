# Implementing a `Monotonic` timer for scheduling

The framework is flexible because it can use any timer which has compare-match and optionally
supporting overflow interrupts for scheduling.
The single requirement to make a timer usable with RTIC is implementing the
[`rtic_monotonic::Monotonic`] trait.

Implementing time counting that supports large time spans is generally **difficult**, in RTIC 0.5
implementing time handling was a common problem.
Moreover, the relation between time and timers used for scheduling was difficult to understand.

For RTIC 1.0 we instead assume the user has a time library, e.g. [`fugit`] or [`embedded_time`],
as the basis for all time-based operations when implementing `Monotonic`.
This makes it much easier to correctly implement the `Monotonic` trait allowing the use of
almost any timer in the system for scheduling.

The trait documents the requirements for each method,
and for inspiration here is a list of `Monotonic` implementations:

- [`STM32F411 series`], implemented for the 32-bit timers
- [`Nordic nRF52 series Timer`], implemented for the 32-bit timers
- [`Nordic nRF52 series RTC`], implemented for the RTCs
- [`Systick based`], runs at a fixed interrupt (tick) rate - with some overhead but simple and with support for large time spans
- [`DWT and Systick based`], a more efficient (tickless) implementation - requires both `SysTick` and `DWT`, supports both high resolution and large time spans

If you know of more implementations feel free to add them to this list.

[`rtic_monotonic::Monotonic`]: https://docs.rs/rtic-monotonic/
[`fugit`]: https://docs.rs/fugit/
[`embedded_time`]: https://docs.rs/embedded_time/
[`STM32F411 series`]: https://github.com/kalkyl/f411-rtic/blob/a696fce7d6d19fda2356c37642c4d53547982cca/src/mono.rs
[`Nordic nRF52 series Timer`]: https://github.com/kalkyl/nrf-play/blob/47f4410d4e39374c18ff58dc17c25159085fb526/src/mono.rs
[`Nordic nRF52 series RTC`]: https://gist.github.com/korken89/fe94a475726414dd1bce031c76adc3dd
[`Systick based`]: https://github.com/rtic-rs/systick-monotonic
[`DWT and Systick based`]: https://github.com/rtic-rs/dwt-systick-monotonic
