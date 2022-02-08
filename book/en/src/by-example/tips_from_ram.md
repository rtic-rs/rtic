# Running tasks from RAM

The main goal of moving the specification of RTIC applications to attributes in
RTIC v0.4.0 was to allow inter-operation with other attributes. For example, the
`link_section` attribute can be applied to tasks to place them in RAM; this can
improve performance in some cases.

> **IMPORTANT**: In general, the `link_section`, `export_name` and `no_mangle`
> attributes are powerful but also easy to misuse. Incorrectly using any of
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
{{#include ../../../../examples/ramfunc.rs}}
```

Running this program produces the expected output.

``` console
$ cargo run --target thumbv7m-none-eabi --example ramfunc
{{#include ../../../../ci/expected/ramfunc.run}}
```

One can look at the output of `cargo-nm` to confirm that `bar` ended in RAM
(`0x2000_0000`), whereas `foo` ended in Flash (`0x0000_0000`).

``` console
$ cargo nm --example ramfunc --release | grep ' foo::'
{{#include ../../../../ci/expected/ramfunc.run.grep.foo}}
```

``` console
$ cargo nm --example ramfunc --release | grep ' bar::'
{{#include ../../../../ci/expected/ramfunc.run.grep.bar}}
```
