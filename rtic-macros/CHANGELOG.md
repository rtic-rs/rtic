# Change Log

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

For each category, *Added*, *Changed*, *Fixed* add new entries at the top!

## [Unreleased]

- Unstable support for ESP32-C6
- Fix: supoort edition 2024 `unsafe(link_section)` attribute
- Adapt `slic` backends to new version with `mecall`
- Allow software tasks to be diverging (return `!`) and give them `'static` context.

## [v2.1.1] - 2024-12-06

### Changed

- Replace `proc-macro-error` with `proc-macro-error2`
- Fix codegen emitting unqualified `Result`
- Improve error output for prios > dispatchers

### Fixed

- Fix interrupt handlers when targeting esp32c3 and using latest version of esp-hal
- Do not limit async priority with `NVIC_PRIO_BITS` when targeting esp32c3

## [v2.1.0] - 2024-02-27

### Added

- Unstable support for RISC-V targets compatible with `riscv-slic`
- RTIC v2 now works on stable.
- Unstable ESP32-C3 support.

### Changed

- Upgraded from syn 1.x to syn 2.x

## [v2.0.1] - 2023-07-25

### Added

- `init` and `idle` can now be externed.

### Fixed

- Support new TAIT syntax requirement.

## [v2.0.0] - 2023-05-31

- Initial v2 release
