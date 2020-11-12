# Types, Send and Sync

Every function within the `app` module has a `Context` structure as its
first parameter. All the fields of these structures have predictable,
non-anonymous types so you can write plain functions that take them as arguments.

The API reference specifies how these types are generated from the input. You
can also generate documentation for you binary crate (`cargo doc --bin <name>`);
in the documentation you'll find `Context` structs (e.g. `init::Context` and
`idle::Context`).

The example below shows the different types generates by the `app` attribute.

``` rust
{{#include ../../../../examples/types.rs}}
```

## `Send`

[`Send`] is a marker trait for "types that can be transferred across thread
boundaries", according to its definition in `core`. In the context of RTIC the
`Send` trait is only required where it's possible to transfer a value between
tasks that run at *different* priorities. This occurs in a few places: in
message passing, in shared resources and in the initialization of late
resources.

[`Send`]: https://doc.rust-lang.org/core/marker/trait.Send.html

The `app` attribute will enforce that `Send` is implemented where required so
you don't need to worry much about it. Currently all types that are passed need
to be `Send` in RTIC, however this restriction might be relaxed in the future.

## `Sync`

Similarly, [`Sync`] is a marker trait for "types for which it is safe to share
references between threads", according to its definition in `core`. In the
context of RTIC the `Sync` trait is only required where it's possible for two,
or more, tasks that run at different priorities and may get a shared reference
(`&-`) to a resource. This only occurs with shared access (`&-`) resources.

[`Sync`]: https://doc.rust-lang.org/core/marker/trait.Sync.html

The `app` attribute will enforce that `Sync` is implemented where required but
it's important to know where the `Sync` bound is not required: shared access
(`&-`) resources contended by tasks that run at the *same* priority.

The example below shows where a type that doesn't implement `Sync` can be used.

``` rust
{{#include ../../../../examples/not-sync.rs}}
```
