# Change Log

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

## [v0.3.0] - 2018-01-15

### Added

- [feat] `&'static mut` references can be safely created by assigning resources to `init`. See the
  `init.resources` section of the `app!` macro documentation and the `safe-static-mut-ref` example
  for details.

### Changed

- [breaking-change] svd2rust dependency has been bumped to v0.12.0

- [breaking-change] resources assigned to tasks, or to idle, that were not declared in the top
  `resources` field generate compiler errors. Before these were assumed to be peripherals, that's no
  longer the case.

- [breaking-change] the layout of `init::Peripherals` has changed. This struct now has two fields:
  `core` and `device`. The value of the `core` field is a struct that owns all the core peripherals
  of the device and the value of the `device` field is a struct that owns all the device specific
  peripherals of the device.

## [v0.2.2] - 2017-11-22

### Added

- Support for runtime initialized resources ("late" resources).

## [v0.2.1] - 2017-07-29

### Fixed

- Link to `app!` macro documentation.

## [v0.2.0] - 2017-07-29

### Added

- The `app!` macro, a macro to declare the tasks and resources of an
  application.

- The `Resource` trait, which is used to write generic code that deals with
  resources.

- Support for system handlers like SYS_TICK.

### Changed

- [breaking-change] The signature of the `atomic` function has changed.

- [breaking-change] The threshold token has become a concrete type and lost its
  `raise` method.

### Removed

- [breaking-change] The `tasks!` and `peripherals!` macros.

- [breaking-change] The ceiling and priority tokens.

- [breaking-change] The `Local`, `Resource` and `Peripheral` structs.

- [breaking-change] The traits related to type level integers.

## [v0.1.1] - 2017-06-05

### Changed

- `peripherals!`: The `register_block` field is now optional

## v0.1.0 - 2017-05-09

- Initial release

[Unreleased]: https://github.com/japaric/cortex-m-rtfm/compare/v0.3.0...HEAD
[v0.3.0]: https://github.com/japaric/cortex-m-rtfm/compare/v0.2.2...v0.3.0
[v0.2.2]: https://github.com/japaric/cortex-m-rtfm/compare/v0.2.1...v0.2.2
[v0.2.1]: https://github.com/japaric/cortex-m-rtfm/compare/v0.2.0...v0.2.1
[v0.2.0]: https://github.com/japaric/cortex-m-rtfm/compare/v0.1.1...v0.2.0
[v0.1.1]: https://github.com/japaric/cortex-m-rtfm/compare/v0.1.0...v0.1.1
