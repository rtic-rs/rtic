# Change Log

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

For each category, *Added*, *Changed*, *Fixed* add new entries at the top!

## [Unreleased]

### Added

- `should_dequeue` to the `Monotonic` trait to handle bugged timers

### Changed

### Fixed

- If the queue was non-empty and a new instant was added that was earlier than `head`, then the queue would no pend the monotonic handler. This would cause the new `head` to be dequeued at the wrong time.

## [v1.0.0] - 2023-xx-xx
