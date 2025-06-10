# rtic-time

Basic definitions and utilities that can be used to keep track of time.

[![crates.io](https://img.shields.io/crates/v/rtic-time)](https://crates.io/crates/rtic-time)
[![docs.rs](https://docs.rs/rtic-time/badge.svg)](https://docs.rs/rtic-time)
[![matrix](https://img.shields.io/matrix/rtic:matrix.org)](https://matrix.to/#/#rtic:matrix.org)


## Content

The main contribution of this crate is to define the [`Monotonic`](https://docs.rs/rtic-time/latest/rtic_time/trait.Monotonic.html) trait. It serves as a standardized interface for libraries to interact with the system's monotonic timers.

Additionally, this crate provides tools and utilities that help with implementing monotonic timers.

## Implementations of the `Monotonic` trait

Check the HAL crate of your device: it might already contain an implementation.

For reference implementations of [`Monotonic`](https://docs.rs/rtic-time/latest/rtic_time/trait.Monotonic.html)
on various hardware, see [`rtic-monotonics`](https://docs.rs/rtic-monotonics/).

## RTIC v1 uses [`rtic-monotonic`](https://github.com/rtic-rs/rtic-monotonic) instead

## Chat

Join us and talk about RTIC in the [Matrix room][matrix-room].

Weekly meeting minutes can be found over at [RTIC HackMD][hackmd].

[matrix-room]: https://matrix.to/#/#rtic:matrix.org
[hackmd]: https://rtic.rs/meeting
