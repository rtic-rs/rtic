# Change Log

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

For each category, *Added*, *Changed*, *Fixed* add new entries at the top!

## [Unreleased]

### Added

- nRF52xxx, nRF9160, nRF5340 Timer and RTC monotonics
- Interrupt tokens for `Systick` and `rp2040` to make sure an interrupt handler exists

### Changed

### Fixed

- Unmask the `rp2040` interrupt
- Use `$crate` and fully qualified paths in macros 

## [v1.0.0] - 2023-05-31
