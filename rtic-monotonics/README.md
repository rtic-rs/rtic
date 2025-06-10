[![crates.io](https://img.shields.io/crates/v/rtic-monotonics.svg)](https://crates.io/crates/rtic-monotonics)
[![crates.io](https://img.shields.io/crates/d/rtic-monotonics.svg)](https://crates.io/crates/rtic-monotonics)

# `rtic-monotonics`

> Reference implementations of the Real-Time Interrupt-driven Concurrency (RTIC) Monotonics timers

Uses [`rtic-time`](https://github.com/rtic-rs/rtic/tree/master/rtic-time) defined [`Monotonic`](https://docs.rs/rtic-time/latest/rtic_time/trait.Monotonic.html) trait.

`rtic-monotonics` is for RTIC v2.

For RTIC v1 see [`rtic-monotonic`](https://github.com/rtic-rs/rtic-monotonic)

## [Documentation](https://docs.rs/rtic-monotonics)

[RTIC book: chapter on monotonics](https://rtic.rs/2/book/en/by-example/delay.html)

### [Changelog `rtic-monotonics`](https://github.com/rtic-rs/rtic/blob/master/rtic-monotonics/CHANGELOG.md)

## Supported Platforms

The following microcontroller families feature efficient monotonics using peripherals.
Refer to the [crate documentation](https://docs.rs/rtic-monotonics) for more details.

- RP2040
- i.MX RT
- nRF
- ATSAMD

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
