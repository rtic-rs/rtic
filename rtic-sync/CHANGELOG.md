# Change Log

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

For each category, _Added_, _Changed_, _Fixed_ add new entries at the top!

## [Unreleased]

### Added

### Changed

### Fixed

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
