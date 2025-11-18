# Change Log

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

For each category, *Added*, *Changed*, *Fixed* add new entries at the top!

## [Unreleased]

### Added

### Changed

### Fixed

## v1.1.0 - 2025-06-22

- Fix minor unsoundnes in `Link::remove_from_list`.

### Added

- New safe `WaitQueue::wait_until` method.

### Changed

### Fixed

## [v1.0.1](https://github.com/rtic-rs/rtic/commit/2b2208e217a96086696bd6f36cff2a6cd4c4ac9f)

- `portable-atomic` used as a drop in replacement for `core::sync::atomic` in code and macros. `portable-atomic` imported with `default-features = false`, as we do not require CAS.

## [v1.0.0](https://github.com/rtic-rs/rtic/commit/e65e532c2a342f77080ac6fc8e5be11aa7d82575)(https://github.com/rtic-rs/rtic/commit/c3884e212c36d2a9cf260b1d9ae37c92b91ea73d) - 2023-05-31
