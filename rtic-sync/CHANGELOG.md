# Change Log

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

For each category, _Added_, _Changed_, _Fixed_ add new entries at the top!

## [Unreleased]

### Changed

- Un-hide lifetimes of output type in `Signal::split` to resolve a new warning.

## [v](https://github.com/rtic-rs/rtic/commit/fe77b4538d6cd506d1a18bdc9e17216dc61881db)1.4.0 - 2025-06-22

### Added

- Add `arbiter::{i2c, spi}::BlockingArbiterDevice` which allows sharing of `embedded_hal` (non-async) buses. This also helps during initialization of RTIC apps as you can use the arbiter while in `init`. After initialization is complete, convert an `BlockingArbiterDevice` into an `ArbiterDevice` using `BlockingArbiterDevice::into_non_blocking()`.

### Fixed

- Avoid a critical section when a `send`-link is popped and when returning `free_slot`.

### Changed

- Actually drop items left over in `Channel` on drop of `Receiver`.
- Allow for `split()`-ing a channel more than once without immediately panicking.
- Add `loom` support.
- Avoid a critical section when a `send`-link is popped and when returning `free_slot`.
- Don't force `Signal` import when using `make_signal` macro
- Update `make_signal`'s documentation to match `make_channel`'s

## [v1.3.2](https://github.com/rtic-rs/rtic/commit/daff0c2913ba5c8c3975313314e531e00a620732) - 2025-03-16

### Fixed

- Improve handling of free slots for `send` by explicitly writing the free slot to the awoken future.
- Fix all known instances of #780

## [v1.3.1](https://github.com/rtic-rs/rtic/commit/bac77de9bc5249a8d4e34c816bb94f5945fb1f58) - 2025-03-12

### Fixed

- Fix [#780]

[#780]: https://github.com/rtic-rs/rtic/issues/780

## [v1.3.0](https://github.com/rtic-rs/rtic/commit/4a23c8d6da918b2ddd5a6b694b584fd2737833bb) - 2024-05-01

### Changed

- Unstable features are now stable, the feature flag `unstable` is removed.
- Update `embedded-hal-bus` to 0.2

### Added

- `defmt v0.3` derives added and forwarded to `embedded-hal(-x)` crates.
- signal structure

## [v1.2.0](https://github.com/rtic-rs/rtic/commit/f69ecb05a95fd7c2906d060c1548291052dba6bd) - 2024-01-10

### Changed

- Using `embedded-hal` 1.0.

### Fixed

- `make_channel` now accepts `Type` expressions instead of only `TypePath` expressions.

## [v1.1.1](https://github.com/rtic-rs/rtic/commit/1622f6b953c93c3a680769636b60733f281f1ac0) - 2023-12-04

### Fixed

- Fix features for `docs.rs`

## [v1.1.0](https://github.com/rtic-rs/rtic/commit/ea8de913d7e7265b13edee779e4ab614a227bef2) - 2023-12-04

### Added

- `arbiter::spi::ArbiterDevice` for sharing SPI buses using `embedded-hal-async` traits.
- `arbiter::i2c::ArbiterDevice` for sharing I2C buses using `embedded-hal-async` traits.

## [v1.0.3](https://github.com/rtic-rs/rtic/commit/2b2208e217a96086696bd6f36cff2a6cd4c4ac9f)

- `portable-atomic` used as a drop in replacement for `core::sync::atomic` in code and macros. `portable-atomic` imported with `default-features = false`, as we do not require CAS.

## [v1.0.2](https://github.com/rtic-rs/rtic/commit/adfe33f5976991a2d957c9e5f209904d46eb934a) - 2023-08-29

### Fixed

- `make_channel` no longer requires the user crate to have `critical_section` in scope

## [v1.0.1](https://github.com/rtic-rs/rtic/commit/db18c00c00deb146478de1b0f94f8181300c47ce) - 2023-06-14

### Fixed

- `make_channel` could be UB

## [v1.0.0](https://github.com/rtic-rs/rtic/commit/c3884e212c36d2a9cf260b1d9ae37c92b91ea73d) - 2023-05-31 - yanked

- Initial release
