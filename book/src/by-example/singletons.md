# Singletons

The `app` attribute is aware of [`owned-singleton`] crate and its [`Singleton`]
attribute. When this attribute is applied to one of the resources the runtime
will perform the `unsafe` initialization of the singleton for you, ensuring that
only a single instance of the singleton is ever created.

[`owned-singleton`]: ../../api/owned_singleton/index.html
[`Singleton`]: ../../api/owned_singleton_macros/attr.Singleton.html

Note that when using the `Singleton` attribute you'll need to have the
`owned_singleton` in your dependencies.

Below is an example that uses the `Singleton` attribute on a chunk of memory
and then uses the singleton instance as a fixed-size memory pool using one of
the [`alloc-singleton`] abstractions.

[`alloc-singleton`]: https://crates.io/crates/alloc-singleton

``` rust
{{#include ../../../examples/singleton.rs}}
```

``` console
$ cargo run --example singleton
{{#include ../../../ci/expected/singleton.run}}```
