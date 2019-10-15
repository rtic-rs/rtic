# Change Log

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

## v0.5.0 - 2019-??-?? (currently in beta pre-release)

### Added

- Experimental support for homogeneous and heterogeneous multi-core
  microcontrollers has been added. Support is gated behind the `homogeneous` and
  `heterogeneous` Cargo features.

### Changed

- [breaking-change][] [RFC 155] "explicit `Context` parameter" has been
  implemented.

[RFC 155]: https://github.com/rtfm-rs/cortex-m-rtfm/issues/155

- [breaking-change][] [RFC 147] "all functions must be safe" has been
  implemented.

[RFC 147]: https://github.com/rtfm-rs/cortex-m-rtfm/issues/147

- All the queues internally used by the framework now use `AtomicU8` indices
  instead of `AtomicUsize`; this reduces the static memory used by the
  framework.

- [breaking-change][] when the `capacity` argument is omitted, the capacity of
  the task is assumed to be `1`. Before, a reasonable (but hard to predict)
  capacity was computed based on the number of `spawn` references the task had.

- [breaking-change][] resources that are appear as exclusive references
  (`&mut-`) no longer appear behind the `Exclusive` newtype.

- [breaking-change][] the `timer-queue` Cargo feature has been removed. The
  `schedule` API can be used without enabling any Cargo feature.

- [breaking-change][] when the `schedule` API is used the type of
  `init::Context.core` changes from `cortex_m::Peripherals` to
  `rtfm::Peripherals`. The fields of `rtfm::Peripherals` do not change when
  Cargo features are enabled.

- [breaking-change][] the monotonic timer used to implement the `schedule` API
  is now user configurable via the `#[app(monotonic = ..)]` argument. IMPORTANT:
  it is now the responsibility of the application author to configure and
  initialize the chosen `monotonic` timer during the `#[init]` phase.

- [breaking-change][] the `peripherals` field is not include in `init::Context`
  by default. One must opt-in using the `#[app(peripherals = ..)]` argument.

- [breaking-change][] the `#[exception]` and `#[interrupt]` attributes have been
  removed. Hardware tasks are now declared using the `#[task(binds = ..)]`
  attribute.

- [breaking-change][] the syntax to declare resources has changed. Instead of
  using a `static [mut]` variable for each resource, all resources must be
  declared in a `Resources` structure.

### Removed

- [breaking-change] the integration with the `owned_singleton` crate has been
  removed. You can use `heapless::Pool` instead of `alloc_singleton`.

- [breaking-change] late resources can no longer be initialized using the assign
  syntax. `init::LateResources` is the only method to initialize late resources.
  See [PR #140] for more details.

[PR #140]: https://github.com/rtfm-rs/cortex-m-rtfm/pull/140

## [v0.4.3] - 2019-04-21

### Changed

- Checking that the specified priorities are supported by the target device is
  now done at compile time.

### Fixed

- Building this crate with the "nightly" feature and a recent compiler has been
  fixed.

## [v0.4.2] - 2019-02-27

### Added

- `Duration` now has an `as_cycles` method to get the number of clock cycles
  contained in it.

- An opt-in "nightly" feature that reduces static memory usage, shortens
  initialization time and reduces runtime overhead has been added. To use this
  feature you need a nightly compiler!

- [RFC 128] has been implemented. The `exception` and `interrupt` have gained a
  `binds` argument that lets you give the handler an arbitrary name. For
  example:

[RFC 128]: https://github.com/rtfm-rs/cortex-m-rtfm/issues/128

``` rust
// on v0.4.1 you had to write
#[interrupt]
fn USART0() { .. }

// on v0.4.2 you can write
#[interrupt(binds = USART0)]
fn on_new_frame() { .. }
```

### Changed

- Builds are now reproducible. `cargo build; cargo clean; cargo build` will
  produce binaries that are exactly the same (after `objcopy -O ihex`). This
  wasn't the case before because we used randomly generated identifiers for
  memory safety but now all the randomness is gone.

### Fixed

- Fixed a `non_camel_case_types` warning that showed up when using a recent
  nightly.

- Fixed a bug that allowed you to enter the `capacity` and `priority` arguments
  in the `task` attribute more than once. Now all arguments can only be stated
  once in the list, as it should be.

## [v0.4.1] - 2019-02-12

### Added

- The RTFM book has been translated to Russian. You can find the translation
  online at https://japaric.github.io/cortex-m-rtfm/book/ru/

- `Duration` now implements the `Default` trait.

### Changed

- [breaking-change][] [soundness-fix] `init` can not contain any early return as
  that would result in late resources not being initialized and thus undefined
  behavior.

- Use an absolute link to the book so it works when landing from crates.io
  documentation page

- The initialization function can now be written as `fn init() ->
  init::LateResources` when late resources are used. This is preferred over the
  old `fn init()` form. See the section on late resources (resources chapter) in
  the book for more details.

### Fixed

- `#[interrupt]` and `#[exception]` no longer produce warnings on recent nightlies.

## [v0.4.0] - 2018-11-03 - YANKED

Yanked due to a soundness issue in `init`; the issue has been mostly fixed in v0.4.1.

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

[Unreleased]: https://github.com/rtfm-rs/cortex-m-rtfm/compare/v0.4.3...HEAD
[v0.4.3]: https://github.com/rtfm-rs/cortex-m-rtfm/compare/v0.4.2...v0.4.3
[v0.4.2]: https://github.com/rtfm-rs/cortex-m-rtfm/compare/v0.4.1...v0.4.2
[v0.4.1]: https://github.com/rtfm-rs/cortex-m-rtfm/compare/v0.4.0...v0.4.1
[v0.4.0]: https://github.com/rtfm-rs/cortex-m-rtfm/compare/v0.3.4...v0.4.0
[v0.3.4]: https://github.com/rtfm-rs/cortex-m-rtfm/compare/v0.3.3...v0.3.4
[v0.3.3]: https://github.com/rtfm-rs/cortex-m-rtfm/compare/v0.3.2...v0.3.3
[v0.3.2]: https://github.com/rtfm-rs/cortex-m-rtfm/compare/v0.3.1...v0.3.2
[v0.3.1]: https://github.com/rtfm-rs/cortex-m-rtfm/compare/v0.3.0...v0.3.1
[v0.3.0]: https://github.com/rtfm-rs/cortex-m-rtfm/compare/v0.2.2...v0.3.0
[v0.2.2]: https://github.com/rtfm-rs/cortex-m-rtfm/compare/v0.2.1...v0.2.2
[v0.2.1]: https://github.com/rtfm-rs/cortex-m-rtfm/compare/v0.2.0...v0.2.1
[v0.2.0]: https://github.com/rtfm-rs/cortex-m-rtfm/compare/v0.1.1...v0.2.0
[v0.1.1]: https://github.com/rtfm-rs/cortex-m-rtfm/compare/v0.1.0...v0.1.1
