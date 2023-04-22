# Implementing a `Monotonic` timer for scheduling

The framework is flexible because it can use any timer which has compare-match and optionally supporting overflow interrupts for scheduling. The single requirement to make a timer usable with RTIC is implementing the `rtic-time::Monotonic` trait.

For RTIC 1.0 and 2.0 we instead assume the user has a time library, e.g. [`fugit`], as the basis for all time-based operations when implementing `Monotonic`. These libraries make it much easier to correctly implement the `Monotonic` trait, allowing the use of almost any timer in the system for scheduling.

The trait documents the requirements for each method. There are reference implementations available in [`rtic-monotonics`](https://github.com/rtic-rs/rtic/tree/master/rtic-monotonics/src) that can be used for inspriation. 

- [`Systick based`], runs at a fixed interrupt (tick) rate - with some overhead but simple and provides support for large time spans
- [`RP2040 Timer`], a "proper" implementation with support for waiting for long periods without interrupts. Clearly demonstrates how to use the `TimerQueue` to handle scheduling.
- [`nRF52 timers`] implements monotonic & Timer Queue for the RTC and normal timers in nRF52's

## Contributing

Contributing new implementations of `Monotonic` can be done in multiple ways:
* Implement the trait behind a feature flag in [`rtic-monotonics`], and create a PR for them to be included in the main RTIC repository. This way, the implementations of are in-tree, and RTIC can guarantee their correctness, and can update them in the case of a new release.
* Implement the changes in an external repository.

<!-- TODO: remove 1.0.x examples? -->
# V1.0.x

Here is a list of `Monotonic` implementations for RTIC 1.0:

- [`STM32F411 series`], implemented for the 32-bit timers
- [`Nordic nRF52 series Timer`], implemented for the 32-bit timers
- [`Nordic nRF52 series RTC`], implemented for the RTCs
- [`DWT and Systick based`], a more efficient (tickless) implementation - requires both `SysTick` and `DWT`, supports both high resolution and large time spans

If you know of more implementations feel free to add them to this list.

[`rtic_time::Monotonic`]: https://docs.rs/rtic_time/
[`fugit`]: https://docs.rs/fugit/
[`STM32F411 series`]: https://github.com/kalkyl/f411-rtic/blob/a696fce7d6d19fda2356c37642c4d53547982cca/src/mono.rs
[`Nordic nRF52 series Timer`]: https://github.com/kalkyl/nrf-play/blob/47f4410d4e39374c18ff58dc17c25159085fb526/src/mono.rs
[`Nordic nRF52 series RTC`]: https://gist.github.com/korken89/fe94a475726414dd1bce031c76adc3dd
[`Systick based`]: https://github.com/rtic-monotonics
[`DWT and Systick based`]: https://github.com/rtic-rs/dwt-systick-monotonic
[`rtic-monotonics`]:  https://github.com/rtic-rs/rtic/blob/master/rtic-monotonics
[`RP2040 Timer`]: https://github.com/rtic-rs/rtic/blob/master/rtic-monotonics/src/rp2040.rs
[`nRF52 timers`]: https://github.com/rtic-rs/rtic/blob/master/rtic-monotonics/src/nrf.rs