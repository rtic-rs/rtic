# Change Log

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

For each category, *Added*, *Changed*, *Fixed* add new entries at the top!

## Unreleased

### Changed

- Panic if STM32 prescaler value would overflow
- Moved away from `portable-atomic` due to it's spinlock design

### Added

- Cortex-M `systick` can be configured with its external clock source

## v2.1.0 - 2025-06-22

### Changed

- Updated esp32c3 dependency to v0.28.0
- Updated esp32c3 dependency to v0.27.0

### Added

- `SYSTIMER` based monotonic for the ESP32-C6

## v2.0.3 - 2024-10-23

### Added

- RP235x support

### Changed

- Updated esp32c3 dependency to v0.26.0
- Update `esp32c3` dependency

### Fixed

- STM32: Make initialization more deterministic
- STM32: Fix race condition that caused missed interrupts

## v2.0.2 - 2024-07-05

### Added
- `SYSTIMER` based monotonic for the ESP32-C3

### Fixed

- Fix `stm32` monotonic for timer peripherals with only two clock compare modules

## v2.0.1 - 2024-06-02

### Changed

- Make monotonics created with their respective macros public

## v2.0.0 - 2024-05-29

### Changed

- Replace `atomic-polyfill` with `portable-atomic`
- Rework all timers based on `rtic-time 2.0.0`
- Most timer tick rates are now configurable
- Tweak `build.rs` to avoid warnings in Nightly 1.78+
- Removed unused `rust-toolchain.toml`
- RP2040 PAC 0.6 support

## v1.5.0 - 2024-01-10

### Changed

- Using `embedded-hal` 1.0.

## v1.4.1 - 2023-12-06

### Fixed

- Fix race condition in `nrf::timer`.
- Fix race condition in `nrf::rtc`.
- Fix errata in `nrf::rtc`.
- Add internal counter integrity check to all half-period based monotonics.
- Apply race condition fixes from `rtic-time`.

## v1.4.0 - 2023-12-04

### Fixed

- **Soundness fix:** Monotonics did not wait long enough in `Duration` based delays.

### Changed

- Bump `rtic-time`

## v1.3.0 - 2023-11-08

### Added

- i.MX RT support

### Fixed

- Fix STM32 rollover race condition
- Fix STM32 support for other chip families

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
