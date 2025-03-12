# Change Log

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

For each category, _Added_, _Changed_, _Fixed_ add new entries at the top!

## [Unreleased]

### Changed

- Unstable features are now stable, the feature flag `unstable` is removed.
- Update `embedded-hal-bus` to 0.2

### Added

- `defmt v0.3` derives added and forwarded to `embedded-hal(-x)` crates.
- signal structure

### Fixed
- Fix [#780]

[#780]: https://github.com/rtic-rs/rtic/issues/780

## v1.2.0 - 2024-01-10

### Changed

- Using `embedded-hal` 1.0.

### Fixed

- `make_channel` now accepts `Type` expressions instead of only `TypePath` expressions.

## v1.1.1 - 2023-12-04

### Fixed

- Fix features for `docs.rs`

## v1.1.0 - 2023-12-04

### Added

- `arbiter::spi::ArbiterDevice` for sharing SPI buses using `embedded-hal-async` traits.
- `arbiter::i2c::ArbiterDevice` for sharing I2C buses using `embedded-hal-async` traits.

## v1.0.3

- `portable-atomic` used as a drop in replacement for `core::sync::atomic` in code and macros. `portable-atomic` imported with `default-features = false`, as we do not require CAS.

## v1.0.2 - 2023-08-29

### Fixed

- `make_channel` no longer requires the user crate to have `critical_section` in scope

## v1.0.1 - 2023-06-14

### Fixed

- `make_channel` could be UB

## v1.0.0 - 2023-05-31 - yanked

- Initial release
