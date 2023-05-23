# RTIC vs. Embassy

## Differences

Embassy provides both Hardware Abstraction Layers, and an executor/runtime, while RTIC aims to only provide an execution framework. For example, embassy provides `embassy-stm32` (a HAL), and `embassy-executor` (an executor). On the other hand, RTIC provides the framework in the form of [`rtic`], and the user is responsible for providing a PAC and HAL implementation (generally from the [`stm32-rs`] project).

Additionally, RTIC aims to provide exclusive access to resources on as low a level of possible, ideally guarded by some form of hardware protection. This allows for access to hardware while not necessarily requiring locking mechanisms on the software level.

## Mixing use of Embassy and RTIC

Since most Embassy and RTIC libraries are runtime agnostic, many details from one project can be used in the other. For example, using [`rtic-monotonics`] in an `embassy-executor` powered project works, and using [`embassy-sync`] (though [`rtic-sync`] is recommended) in an RTIC project works.

[`stm32-rs`]: https://github.com/stm32-rs
[`rtic`]: https://docs.rs/rtic/latest/rtic/
[`rtic-monotonics`]: https://docs.rs/rtic-monotonics/latest/rtic_monotonics/
[`embassy-sync`]: https://docs.rs/embassy-sync/latest/embassy_sync/
[`rtic-sync`]: https://docs.rs/rtic-sync/latest/rtic_sync/