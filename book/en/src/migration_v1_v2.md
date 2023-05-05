# Migrating from v1.0.x to v2.0.0

Migrating a project from RTIC `v1.0.x` to `v2.0.0` involves the following steps:

1. `v2.0.0` requires [`#![type_alias_impl_trait]`](https://github.com/rust-lang/rust/issues/63063) and Rust Nightly.
2. Migrating from the monotonics included in `v1.0.x` to `rtic-time` and `rtic-monotonics`, replacing `spawn_after`, `spawn_at`.
3. Software tasks are now required to be `async`, and using them correctly.
4. Understanding and using data types provided by `rtic-sync`.

For a detailed description of the changes, refer to the subchapters.

If you wish to see a code example of changes required, you can check out [the full example migration page](./migration_v1_v2/complete_example.md).

#### TL;DR (Too Long; Didn't Read)
1. Add `#![type_alias_impl_trait]` to your crate, and use `cargo +nightly`.
2. Instead of `spawn_after` and `spawn_at`, you now use the `async` functions `delay`, `delay_until` (and related) with impls provided by `rtic-monotonics`.
3. Software tasks _must_ be `async fn`s now. Not returning from a task is allowed so long as there is an `await` in the task. You can still `lock` shared resources.
4. Use `rtic_sync::Arbiter` to `await` access to a shared resource, and `rtic-channel` to communicate between tasks instead of `spawn`-ing new ones.