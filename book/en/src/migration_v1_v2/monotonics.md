# Migrating to `rtic-monotonics`

In previous versions of `rtic`, monotonics were an integral, tightly coupled part of the `#[rtic::app]`. In this new version, [`rtic-monotonics`] provides them in a more decoupled way.

The `#[monotonic]` attribute is no longer used. Instead, you use a `create_X_token` from [`rtic-monotonics`]. An invocation of this macro returns an interrupt registration token, which can be used to construct an instance of your desired monotonic.

`spawn_after` and `spawn_at` are no longer available. Instead, you use the async functions `delay` and `delay_until` provided by ipmlementations of the `rtic_time::Monotonic` trait, available through [`rtic-monotonics`].

Check out the [code example](./complete_example.md) for an overview of the required changes.

For more information on current monotonic implementations, see [the `rtic-monotonics` documentation](https://docs.rs/rtic-monotonics), and [the examples](https://github.com/rtic-rs/rtic/tree/master/examples).

[`rtic-monotonics`]: ghttps://github.com/rtic/rtic-monotonics