# Change Log

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

-------------------

For each category, *Added*, *Changed*, *Fixed* add **new entries at the top**!

Example:

```
### Changed

- My new entry goes here
- Previous change

```
-------------------

## [Unreleased]

- Added public `waker` constructor to the executor.

## [v2.1.2] - 2024-12-06

### Changed

- Updated esp32c3 dependency to v0.26.0
- Updated esp32c3 dependency to v0.25.0
- Replace `atomic-polyfill` with `portable-atomic`
- Updated esp32c3 dependency to v0.22.0
- Use `riscv-slic` from `crates.io`
- Remove unused dependency `rtic-monotonics`

## [v2.1.1] - 2024-03-13

### Fixed

- **Soundness fix:** `thumbv6` was subject to race in source mask.

## [v2.1.0] - 2024-02-27

### Added

- Unstable support for RISC-V targets compatible with `riscv-slic`
- Unstable support for ESP32-C3

### Fixed

- **Soundness fix:** `thumbv7` was subject to priority inversion.
- **Soundness fix:** Monotonics did not wait long enough in `Duration` based delays.
  This is not directly a change for `rtic`, but required bumping the minimal version of `rtic-monotonics`.

### Changed

- RTIC v2 now works on stable.

## [v2.0.1] - 2023-07-25

### Added

- Allow `#[init]` and `#[idle]` to be defined externally

### Fixed

- Support new TAIT syntax requirement.

### Changed

- `cortex-m` set as an optional dependency
- Moved `cortex-m`-related utilities from `rtic/lib.rs` to `rtic/export.rs`
- Make async task priorities start at 0, instead of 1, to always start at the lowest priority

## [v2.0.0] - 2023-05-31

- v2 is a massive change, refer to the book for more details

## [v1.1.4] - 2023-02-26

### Added

- CFG: Support #[cfg] on HW task, cleanup for SW tasks
- CFG: Slightly improved support for #[cfg] on Monotonics
- CI: Check examples also for thumbv8.{base,main}
- Allow custom `link_section` attributes for late resources

### Fixed

- Attempt to handle docs generation enabling `deny(missing_docs)`
- Book: Editorial review
- Use native GHA rustup and cargo
- Distinguish between thumbv8m.base and thumbv8m.main for basepri usage.

### Changed

- Updated dev-dependency cortex-m-semihosting to v0.5
- CI: Updated to setup-python@v4
- CI: Updated to checkout@v3
- Tuned redirect message for rtic.rs/meeting

## [v1.1.3] - 2022-06-23

### Added

### Fixed

- Bump cortex-m-rtic-macros to 1.1.5
- fix ci: use SYST::PTR

#### cortex-m-rtic-macros v1.1.5 - 2022-06-23

- Bump rtic-syntax to 1.0.2

#### cortex-m-rtic-macros v1.1.4 - 2022-05-24

- Fix macros to Rust 2021

#### cortex-m-rtic-macros v1.1.3 - 2022-05-24

- Fix clash with defmt

### Changed

## [v1.1.2] - 2022-05-09

### Added

### Fixed

- Generation of masks for the source masking scheduling for thumbv6

### Changed

## [v1.1.1] - 2022-04-13 - YANKED

### Added

### Fixed

- Fixed `marcro` version

### Changed

## [v1.1.0] - 2022-04-13 - YANKED

### Added

- Improve how CHANGELOG.md merges are handled
- If current $stable and master version matches, dev-book redirects to $stable book
- During deploy stage, merge master branch into current stable IFF cargo package version matches
- Rework branch structure, release/vVERSION
- Cargo clippy in CI
- Use rust-cache Github Action
- Support for NVIC based SPR based scheduling for armv6m.
- CI changelog entry enforcer
- `examples/periodic-at.rs`, an example of a periodic timer without accumulated drift.
- `examples/periodic-at2.rs`, an example of a periodic process with two tasks, with offset timing.
  Here we depict two alternative usages of the timer type, explicit and trait based.
- book: Update `Monotonic` tips.

### Fixed

- Re-export `rtic_core::prelude` as `rtic::mutex::prelude` to allow glob imports + Clippy
- Fix all except `must_use` lints from clippy::pedantic
- Fix dated migration docs for spawn
- Remove obsolete action-rs tool-cache
- Force mdBook to return error codes
- Readded missing ramfunc output to book

### Changed

- Try to detect `target-dir` for rtic-expansion.rs

## [v1.0.0] - 2021-12-25

### Changed

- Bump RTIC dependencies also updated to v1.0.0
- Edition 2021
- Change default `idle` behaviour to be `NOP` instead of `WFI`

## [v0.6.0-rc.4] - 2021-11-09

- Updated to use the new generic `Monotonic` trait

## [v0.6.0-rc.3] - 2021-11-08

### Fixed

- Match rtic-syntax Analysis-struct updates from https://github.com/rtic-rs/rtic-syntax/pull/61

## [v0.6.0-rc.2] - 2021-09-28

- Fixed issue with `cortex_m` being used by the codegen instead of using the `rtic::export::...` which could make an app not compile if Systick is used and the user did not have the cortex-m crate as a dependency

## [v0.6.0-rc.1] - 2021-09-27

- Documentation updates
- Monotonic handlers default to maximum priority instead of minimum (to follow RTIC 0.5)
- Better support for `rust-analyzer`

## [v0.5.9] - 2021-09-27

- Removed the `cortex-m-rt` dependency
- Docs updates

## [v0.5.8] - 2021-08-19

- Feature flag was added to support `cortex-m v0.7.x`
- MSRV raised to 1.38.

## [v0.6.0-alpha.5] - 2021-07-09

### Changed

- The new resources syntax is implemented.

## [v0.5.7] - 2021-07-05

- Backport: "you must enable the rt feature" compile time detection

## [v0.6.0-alpha.4] - 2021-05-27

### Fixed

- Fixed codegen structure to not have issues with local paths
- Default paths for monotonics now work properly
- New `embedded-time` version to `0.11`

## [v0.6.0-alpha.3] - 2021-0X-XX

- Lost in the ether...

## [v0.6.0-alpha.2] - 2021-04-08

### Added

- Cancel and reschedule support to the monotonics

### Fixed

- UB in `spawn_at`
- `#[cfg]` and other attributes now work on hardware tasks
- Type aliases now work in `mod app`

### Changed

- The access to monotonic static methods was for example `MyMono::now()`, and is now `monotonics::MyMono::now()`

## [v0.6.0-alpha.1] - 2021-03-04

### Added

- Support for multi-locks, see `examples/multilock.rs` for syntax.
- New monotonic syntax and support, see `#[monotonic]`

## [v0.5.6] - 2021-03-03

- **Security** Use latest security patched heapless

## [v0.6.0-alpha.0] - 2020-11-14

### Added

- Allow annotating resources to activate special resource locking behaviour.
  - `#[lock_free]`, there might be several tasks with the same priority accessing
    the resource without critical section.
  - `#[task_local]`, there must be only one task, similar to a task local
    resource, but (optionally) set-up by init. This is similar to move.

- Improved ergonomics allowing separation of task signatures to actual implementation in extern block `extern "Rust" { #[task(..)] fn t(..); }`.

### Changed

- [breaking-change] [PR 400] Move dispatchers from extern block to app argument.

[PR 400]: https://github.com/rtic-rs/cortex-m-rtic/pull/400

- [breaking-change] [PR 399] Locking resources are now always required to achieve a symmetric UI.

[PR 399]: https://github.com/rtic-rs/cortex-m-rtic/pull/399

- [breaking-change] [PR 390]  Rework whole spawn/schedule, support `foo::spawn( ... )`,
  `foo::schedule( ... )`.

[PR 390]: https://github.com/rtic-rs/cortex-m-rtic/pull/390

- [breaking-change] [PR 368] `struct Resources` changed to attribute `#[resources]` on a struct.

- [breaking-change] [PR 368] Mod over const, instead of `const APP: () = {` use `mod app {`.

- [breaking-change] [PR 372] Init function always return `LateResources` for a symmetric API.

- [PR 355] Multi-core support was removed to reduce overall complexity.

[PR 368]: https://github.com/rtic-rs/cortex-m-rtic/pull/368
[PR 372]: https://github.com/rtic-rs/cortex-m-rtic/pull/372
[PR 355]: https://github.com/rtic-rs/cortex-m-rtic/pull/355

## [v0.5.5] - 2020-08-27

- Includes the previous soundness fix.
- Fixes wrong use of the `cortex_m` crate which can cause some projects to stop compiling.

## [v0.5.4] - 2020-08-26 - YANKED

- **Soundness fix in RTIC**, it was previously possible to get the `cortex_m::Peripherals` more than once, causing UB.

## [v0.5.3] - 2020-06-12

- Added migration guide from `cortex-m-rtfm` to `cortex-m-rtic`
- No code changes, only a version compatibility release with `cortex-m-rtfm` to ease the transition
for users.

## [v0.5.2] - 2020-06-11

- Using safe `DWT` interface
- Using GitHub Actions now
- Improved CI speed
- Now `main` can be used as function name
- Fixed so one can `cfg`-out resources when using a newer compiler

## [v0.5.1] - 2019-11-19

- Fixed arithmetic wrapping bug in src/cyccntr.rs
  elapsed and duration could cause an internal overflow trap
  on subtraction in debug mode.

- Fixed bug in SysTick implementation where the SysTick could be disabled by
  accident

## [v0.5.0] - 2019-11-14

### Added

- Experimental support for homogeneous and heterogeneous multi-core
  microcontrollers has been added. Support is gated behind the `homogeneous` and
  `heterogeneous` Cargo features.

### Changed

- [breaking-change] [RFC 155] "explicit `Context` parameter" has been
  implemented.

[RFC 155]: https://github.com/rtic-rs/cortex-m-rtic/issues/155

- [breaking-change] [RFC 147] "all functions must be safe" has been
  implemented.

[RFC 147]: https://github.com/rtic-rs/cortex-m-rtic/issues/147

- All the queues internally used by the framework now use `AtomicU8` indices
  instead of `AtomicUsize`; this reduces the static memory used by the
  framework.

- [breaking-change] when the `capacity` argument is omitted, the capacity of
  the task is assumed to be `1`. Before, a reasonable (but hard to predict)
  capacity was computed based on the number of `spawn` references the task had.

- [breaking-change] resources that are appear as exclusive references
  (`&mut-`) no longer appear behind the `Exclusive` newtype.

- [breaking-change] the `timer-queue` Cargo feature has been removed. The
  `schedule` API can be used without enabling any Cargo feature.

- [breaking-change] when the `schedule` API is used the type of
  `init::Context.core` changes from `cortex_m::Peripherals` to
  `rtic::Peripherals`. The fields of `rtic::Peripherals` do not change when
  Cargo features are enabled.

- [breaking-change] the monotonic timer used to implement the `schedule` API
  is now user configurable via the `#[app(monotonic = ..)]` argument. IMPORTANT:
  it is now the responsibility of the application author to configure and
  initialize the chosen `monotonic` timer during the `#[init]` phase.

- [breaking-change] the `peripherals` field is not include in `init::Context`
  by default. One must opt-in using the `#[app(peripherals = ..)]` argument.

- [breaking-change] the `#[exception]` and `#[interrupt]` attributes have been
  removed. Hardware tasks are now declared using the `#[task(binds = ..)]`
  attribute.

- [breaking-change] the syntax to declare resources has changed. Instead of
  using a `static [mut]` variable for each resource, all resources must be
  declared in a `Resources` structure.

### Removed

- [breaking-change] the integration with the `owned_singleton` crate has been
  removed. You can use `heapless::Pool` instead of `alloc_singleton`.

- [breaking-change] late resources can no longer be initialized using the assign
  syntax. `init::LateResources` is the only method to initialize late resources.
  See [PR #140] for more details.

[PR #140]: https://github.com/rtic-rs/cortex-m-rtic/pull/140

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

[RFC 128]: https://github.com/rtic-rs/cortex-m-rtic/issues/128

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

- The RTIC book has been translated to Russian. You can find the translation
  online at https://japaric.github.io/cortex-m-rtic/book/ru/

- `Duration` now implements the `Default` trait.

### Changed

- [breaking-change] [soundness-fix] `init` can not contain any early return as
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

- [breaking-change] The name of the library has changed to `rtic`. The package
  name is still `cortex-m-rtic`.

- [breaking-change] `cortex_m_rtic::set_pending` has been renamed to
  `rtic::pend`.

### Added

- Software tasks, which can be immediately spawn and scheduled to run in the
  future.

- `Instant` and `Duration` API.

- Integration with the [`Singleton`] abstraction.

[`Singleton`]: https://docs.rs/owned-singleton/0.1.0/owned_singleton/

### Removed

- [breaking-change] The `Threshold` token has been removed.

- [breaking-change] The `bkpt` and `wfi` re-exports have been removed.

- [breaking-change] `rtic::atomic` has been removed.

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

[Unreleased]: https://github.com/rtic-rs/rtic/compare/v2.0.1...HEAD
[v2.0.1]: https://github.com/rtic-rs/rtic/compare/v2.0.0...v2.0.1
[v2.0.0]: https://github.com/rtic-rs/rtic/compare/v1.1.4...v2.0.0
[v1.1.4]: https://github.com/rtic-rs/rtic/compare/v1.1.3...v1.1.4
[v1.1.3]: https://github.com/rtic-rs/rtic/compare/v1.1.2...v1.1.3
[v1.1.2]: https://github.com/rtic-rs/rtic/compare/v1.1.1...v1.1.2
[v1.1.1]: https://github.com/rtic-rs/rtic/compare/v1.1.0...v1.1.1
[v1.1.0]: https://github.com/rtic-rs/rtic/compare/v1.0.0...v1.1.0
[v1.0.0]: https://github.com/rtic-rs/rtic/compare/v0.6.0-rc.4...v1.0.0
[v0.6.0-rc.4]: https://github.com/rtic-rs/rtic/compare/v0.6.0-rc.3...v0.6.0-rc.4
[v0.6.0-rc.3]: https://github.com/rtic-rs/rtic/compare/v0.6.0-rc.2...v0.6.0-rc.3
[v0.6.0-rc.2]: https://github.com/rtic-rs/rtic/compare/v0.6.0-rc.1...v0.6.0-rc.2
[v0.6.0-rc.1]: https://github.com/rtic-rs/rtic/compare/v0.6.0-rc.0...v0.6.0-rc.1
[v0.6.0-rc.0]: https://github.com/rtic-rs/rtic/compare/v0.6.0-alpha.5...v0.6.0-rc.0
[v0.6.0-alpha.5]: https://github.com/rtic-rs/rtic/compare/v0.6.0-alpha.4...v0.6.0-alpha.5
[v0.6.0-alpha.4]: https://github.com/rtic-rs/rtic/compare/v0.6.0-alpha.3...v0.6.0-alpha.4
[v0.6.0-alpha.3]: https://github.com/rtic-rs/rtic/compare/v0.6.0-alpha.2...v0.6.0-alpha.3
[v0.6.0-alpha.2]: https://github.com/rtic-rs/rtic/compare/v0.6.0-alpha.1...v0.6.0-alpha.2
[v0.6.0-alpha.1]: https://github.com/rtic-rs/rtic/compare/v0.6.0-alpha.0...v0.6.0-alpha.1
[v0.6.0-alpha.0]: https://github.com/rtic-rs/rtic/compare/v0.5.5...v0.6.0-alpha.0
[v0.5.x unreleased]: https://github.com/rtic-rs/rtic/compare/v0.5.8...v0.5.x
[v0.5.9]: https://github.com/rtic-rs/rtic/compare/v0.5.8...v0.5.9
[v0.5.8]: https://github.com/rtic-rs/rtic/compare/v0.5.7...v0.5.8
[v0.5.7]: https://github.com/rtic-rs/rtic/compare/v0.5.6...v0.5.7
[v0.5.6]: https://github.com/rtic-rs/rtic/compare/v0.5.5...v0.5.6
[v0.5.5]: https://github.com/rtic-rs/rtic/compare/v0.5.4...v0.5.5
[v0.5.4]: https://github.com/rtic-rs/rtic/compare/v0.5.3...v0.5.4
[v0.5.3]: https://github.com/rtic-rs/rtic/compare/v0.5.2...v0.5.3
[v0.5.2]: https://github.com/rtic-rs/rtic/compare/v0.5.1...v0.5.2
[v0.5.1]: https://github.com/rtic-rs/rtic/compare/v0.5.0...v0.5.1
[v0.5.0]: https://github.com/rtic-rs/rtic/compare/v0.4.3...v0.5.0
[v0.4.3]: https://github.com/rtic-rs/rtic/compare/v0.4.2...v0.4.3
[v0.4.2]: https://github.com/rtic-rs/rtic/compare/v0.4.1...v0.4.2
[v0.4.1]: https://github.com/rtic-rs/rtic/compare/v0.4.0...v0.4.1
[v0.4.0]: https://github.com/rtic-rs/rtic/compare/v0.3.4...v0.4.0
[v0.3.4]: https://github.com/rtic-rs/rtic/compare/v0.3.3...v0.3.4
[v0.3.3]: https://github.com/rtic-rs/rtic/compare/v0.3.2...v0.3.3
[v0.3.2]: https://github.com/rtic-rs/rtic/compare/v0.3.1...v0.3.2
[v0.3.1]: https://github.com/rtic-rs/rtic/compare/v0.3.0...v0.3.1
[v0.3.0]: https://github.com/rtic-rs/rtic/compare/v0.2.2...v0.3.0
[v0.2.2]: https://github.com/rtic-rs/rtic/compare/v0.2.1...v0.2.2
[v0.2.1]: https://github.com/rtic-rs/rtic/compare/v0.2.0...v0.2.1
[v0.2.0]: https://github.com/rtic-rs/rtic/compare/v0.1.1...v0.2.0
[v0.1.1]: https://github.com/rtic-rs/rtic/compare/v0.1.0...v0.1.1
