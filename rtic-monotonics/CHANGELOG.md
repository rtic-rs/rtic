# Change Log

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

For each category, *Added*, *Changed*, *Fixed* add new entries at the top!

## Unreleased

## v1.2.0 - 2023-09-19

### Added

- STM32 support.
- `embedded-hal` 1.0.0-rc.1 `DelayUs` support

## v1.1.0 - 2023-08-29

### Added

- Feature `systick-64bit` to get 64-bit backed `TimerInstantU64` instead of `TimerInstantU32` from the SysTick-based monotonic timer

## v1.0.1 - 2023-08-20

### Added

- RP2040 PAC 0.5 support
- nRF52xxx, nRF9160, nRF5340 Timer and RTC monotonics
- Interrupt tokens for `Systick` and `rp2040` to make sure an interrupt handler exists

### Changed

- Bump `embedded-hal-async`

### Fixed

- Unmask the `rp2040` interrupt
- Use `$crate` and fully qualified paths in macros

## v1.0.0 - 2023-05-31
