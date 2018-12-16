# Tips & tricks

## Generics

Resources shared between two or more tasks implement the `Mutex` trait in *all*
contexts, even on those where a critical section is not required to access the
data. This lets you easily write generic code that operates on resources and can
be called from different tasks. Here's one such example:

``` rust
{{#include ../../../examples/generics.rs}}
```

``` console
$ cargo run --example generics
{{#include ../../../ci/expected/generics.run}}```

This also lets you change the static priorities of tasks without having to
rewrite code. If you consistently use `lock`s to access the data behind shared
resources then your code will continue to compile when you change the priority
of tasks.

## Conditional compilation

You can use conditional compilation (`#[cfg]`) on resources (`static [mut]`
items) and tasks (`fn` items). The effect of using `#[cfg]` attributes is that
the resource / task will *not* be injected into the prelude of tasks that use
them (see `resources`, `spawn` and `schedule`) if the condition doesn't hold.

The example below logs a message whenever the `foo` task is spawned, but only if
the program has been compiled using the `dev` profile.

``` rust
{{#include ../../../examples/cfg.rs}}
```

## Running tasks from RAM

The main goal of moving the specification of RTFM applications to attributes in
RTFM v0.4.x was to allow inter-operation with other attributes. For example, the
`link_section` attribute can be applied to tasks to place them in RAM; this can
improve performance in some cases.

> **IMPORTANT**: In general, the `link_section`, `export_name` and `no_mangle`
> attributes are very powerful but also easy to misuse. Incorrectly using any of
> these attributes can cause undefined behavior; you should always prefer to use
> safe, higher level attributes around them like `cortex-m-rt`'s `interrupt` and
> `exception` attributes.
>
> In the particular case of RAM functions there's no
> safe abstraction for it in `cortex-m-rt` v0.6.5 but there's an [RFC] for
> adding a `ramfunc` attribute in a future release.

[RFC]: https://github.com/rust-embedded/cortex-m-rt/pull/100

The example below shows how to place the higher priority task, `bar`, in RAM.

``` rust
{{#include ../../../examples/ramfunc.rs}}
```

Running this program produces the expected output.

``` console
$ cargo run --example ramfunc
{{#include ../../../ci/expected/ramfunc.run}}```

One can look at the output of `cargo-nm` to confirm that `bar` ended in RAM
(`0x2000_0000`), whereas `foo` ended in Flash (`0x0000_0000`).

``` console
$ cargo nm --example ramfunc --release | grep ' foo::'
{{#include ../../../ci/expected/ramfunc.grep.foo}}```

``` console
$ cargo nm --example ramfunc --release | grep ' bar::'
{{#include ../../../ci/expected/ramfunc.grep.bar}}```
