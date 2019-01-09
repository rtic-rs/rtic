# Change Log

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

- Use an absolute link to the book so it works when landing from crates.io
  documentation page

## [v0.4.0] - 2018-11-03

### Changed

- This crate now compiles on stable 1.31.

- [breaking-change] The `app!` macro has been transformed into an attribute. See
  the documentation for details.

- [breaking-change] Applications that use this library must be written using the
  2018 edition.

- [breaking-change] The `Resource` trait has been renamed to `Mutex`.
  `Resource.claim_mut` has been renamed to `Mutex.lock` and its signature has
  changed (no `Threshold` token is required).

- [breaking-change] The name of the library has changed to `rtfm`. The package
  name is still `cortex-m-rtfm`.

- [breaking-change] `cortex_m_rtfm::set_pending` has been renamed to
  `rtfm::pend`.

### Added

- Software tasks, which can be immediately spawn and scheduled to run in the
  future.

- `Instant` and `Duration` API.

- Integration with the [`Singleton`] abstraction.

[`Singleton`]: https://docs.rs/owned-singleton/0.1.0/owned_singleton/

### Removed

- [breaking-change] The `Threshold` token has been removed.

- [breaking-change] The `bkpt` and `wfi` re-exports have been removed.

- [breaking-change] `rtfm::atomic` has been removed.

## [v0.3.4] - 2018-08-27

### Changed

- The documentation link to point to GH pages.

## [v0.3.3] - 2018-08-24

### Fixed

- Compilation with latest nightly

## [v0.3.2] - 2018-04-16

### Added

- Span information to error messages

### Changed

- Some non fatal error messages have become warning messages. For example, specifying an empty list
  of resources now produces a warning instead of a hard error.

## [v0.3.1] - 2018-01-16

### Fixed

- Documentation link

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

[Unreleased]: https://github.com/japaric/cortex-m-rtfm/compare/v0.4.0...HEAD
[v0.4.0]: https://github.com/japaric/cortex-m-rtfm/compare/v0.3.4...v0.4.0
[v0.3.4]: https://github.com/japaric/cortex-m-rtfm/compare/v0.3.3...v0.3.4
[v0.3.3]: https://github.com/japaric/cortex-m-rtfm/compare/v0.3.2...v0.3.3
[v0.3.2]: https://github.com/japaric/cortex-m-rtfm/compare/v0.3.1...v0.3.2
[v0.3.1]: https://github.com/japaric/cortex-m-rtfm/compare/v0.3.0...v0.3.1
[v0.3.0]: https://github.com/japaric/cortex-m-rtfm/compare/v0.2.2...v0.3.0
[v0.2.2]: https://github.com/japaric/cortex-m-rtfm/compare/v0.2.1...v0.2.2
[v0.2.1]: https://github.com/japaric/cortex-m-rtfm/compare/v0.2.0...v0.2.1
[v0.2.0]: https://github.com/japaric/cortex-m-rtfm/compare/v0.1.1...v0.2.0
[v0.1.1]: https://github.com/japaric/cortex-m-rtfm/compare/v0.1.0...v0.1.1
