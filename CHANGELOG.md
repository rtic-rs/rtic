# Change Log

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

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

[Unreleased]: https://github.com/japaric/cortex-m-rtfm/compare/v0.2.2...HEAD
[v0.2.2]: https://github.com/japaric/cortex-m-rtfm/compare/v0.2.1...v0.2.2
[v0.2.1]: https://github.com/japaric/cortex-m-rtfm/compare/v0.2.0...v0.2.1
[v0.2.0]: https://github.com/japaric/cortex-m-rtfm/compare/v0.1.1...v0.2.0
[v0.1.1]: https://github.com/japaric/cortex-m-rtfm/compare/v0.1.0...v0.1.1
