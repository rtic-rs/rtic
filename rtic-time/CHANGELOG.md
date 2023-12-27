# Change Log

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

For each category, *Added*, *Changed*, *Fixed* add new entries at the top!

## Unreleased - v2.0.0


### Added

### Changed
- Full rewrite of the `Monotonic` API.
    - Now split into multiple traits:
        - `Monotonic` - A user-facing trait that defines what the functionality of a monotonic is.
        - `TimerQueueBackend` - The set of functionality a backend must provide in order to be used with the `TimerQueue`.
    - `TimerQueue` is now purely based on ticks and has no concept of real time.
    - The `TimerQueueBasedMonotonic` trait implements a `Monotonic` based on a `TimerQueueBackend`, translating ticks into `Instant` and `Duration`.

### Fixed

- Docs: Rename `DelayUs` to `DelayNs` in docs.

## v1.3.0 - 2024-01-10

### Changed

- Using `embedded-hal` 1.0.

## v1.2.0 - 2023-12-06

### Changed

- Docs: Add sanity check to `half_period_counter` code example
- Deprecate `Monotonic::should_dequeue_check` as it was erroneous

### Fixed

- Fix race condition in `half_period_counter::calculate_now`.
  This sadly required a minor API change.

## v1.1.0 - 2023-12-04

### Added

- `half_period_counter` containing utilities for implementing a half-period-counter based monotonic.
- `should_dequeue_check` to the `Monotonic` trait to handle bugged timers.

### Changed

### Fixed

- **Soundness fix:** `TimerQueue` did not wait long enough in `Duration` based delays. Fixing this sadly required adding a `const TICK_PERIOD` to the `Monotonic` trait, which requires updating all existing implementations.
- If the queue was non-empty and a new instant was added that was earlier than `head`, then the queue would no pend the monotonic handler. This would cause the new `head` to be dequeued at the wrong time.

## v1.0.0 - 2023-05-31
