# Change Log

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

For each category, *Added*, *Changed*, *Fixed* add new entries at the top!

## Unreleased

### Changed
- Panic if STM32 prescaler value would overflow

## v2.1.0 - 2025-06-22

### Changed

- Updated esp32c3 dependency to v0.28.0
- Updated esp32c3 dependency to v0.27.0

### Added

- `SYSTIMER` based monotonic for the ESP32-C6

## [v2.0.3](https://github.com/rtic-rs/rtic/commit/d251ba717393a73e9ea26a34fe738e3baec477d2) - 2024-10-23

### Added

- RP235x support

### Changed

- Updated esp32c3 dependency to v0.26.0
- Update `esp32c3` dependency

### Fixed

- STM32: Make initialization more deterministic
- STM32: Fix race condition that caused missed interrupts

## [v2.0.2](https://github.com/rtic-rs/rtic/commit/f925cbe5061ec4ade77935de4a0a790e7fc3ba7c) - 2024-07-05

### Added
- `SYSTIMER` based monotonic for the ESP32-C3

### Fixed

- Fix `stm32` monotonic for timer peripherals with only two clock compare modules

## [v2.0.1](https://github.com/rtic-rs/rtic/commit/689c4a068eddfe32956c1975cdc241b26d1751da) - 2024-06-02

### Changed

- Make monotonics created with their respective macros public

## [v2.0.0](https://github.com/rtic-rs/rtic/commit/8c23e178f3838bcdd13662a2ffefd39ec144e869) - 2024-05-29

### Changed

- Replace `atomic-polyfill` with `portable-atomic`
- Rework all timers based on `rtic-time 2.0.0`
- Most timer tick rates are now configurable
- Tweak `build.rs` to avoid warnings in Nightly 1.78+
- Removed unused `rust-toolchain.toml`
- RP2040 PAC 0.6 support

## [v1.5.0](https://github.com/rtic-rs/rtic/commit/f69ecb05a95fd7c2906d060c1548291052dba6bd) - 2024-01-10

### Changed

- Using `embedded-hal` 1.0.

## [v1.4.1](https://github.com/rtic-rs/rtic/commit/e53624c26396019849e10374eacaf416b11c4e5a) - 2023-12-06

### Fixed

- Fix race condition in `nrf::timer`.
- Fix race condition in `nrf::rtc`.
- Fix errata in `nrf::rtc`.
- Add internal counter integrity check to all half-period based monotonics.
- Apply race condition fixes from `rtic-time`.

## [v1.4.0](https://github.com/rtic-rs/rtic/commit/ea8de913d7e7265b13edee779e4ab614a227bef2) - 2023-12-04

### Fixed

- **Soundness fix:** Monotonics did not wait long enough in `Duration` based delays.

### Changed

- Bump `rtic-time`

## [v1.3.0](https://github.com/rtic-rs/rtic/commit/4425b76c6f25a782ea2c473adfa99aec1e5795ac) - 2023-11-08

### Added

- i.MX RT support

### Fixed

- Fix STM32 rollover race condition
- Fix STM32 support for other chip families

## [v1.2.0](https://github.com/rtic-rs/rtic/commit/3b8d787a917a7a39b28bea85ba2b3a86539e0852) - 2023-09-19

### Added

- STM32 support.
- `embedded-hal` 1.0.0-rc.1 `DelayUs` support

## [v1.1.0](https://github.com/rtic-rs/rtic/commit/adfe33f5976991a2d957c9e5f209904d46eb934a) - 2023-08-29

### Added

- Feature `systick-64bit` to get 64-bit backed `TimerInstantU64` instead of `TimerInstantU32` from the SysTick-based monotonic timer

## [v1.0.1](https://github.com/rtic-rs/rtic/commit/df66163aceb1128686e9efcf77d6e3e8520f86b3) - 2023-08-20

### Added

- RP2040 PAC 0.5 support
- nRF52xxx, nRF9160, nRF5340 Timer and RTC monotonics
- Interrupt tokens for `Systick` and `rp2040` to make sure an interrupt handler exists

### Changed

- Bump `embedded-hal-async`

### Fixed

- Unmask the `rp2040` interrupt
- Use `$crate` and fully qualified paths in macros

## [v1.0.0](https://github.com/rtic-rs/rtic/commit/c3884e212c36d2a9cf260b1d9ae37c92b91ea73d) - 2023-05-31
