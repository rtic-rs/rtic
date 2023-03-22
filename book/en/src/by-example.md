# RTIC by example

This part of the book introduces the RTIC framework to new users by walking them through examples of increasing complexity.

All examples in this part of the book are accessible at the
[GitHub repository][repoexamples].
The examples are runnable on QEMU (emulating a Cortex M3 target),
thus no special hardware required to follow along.

[repoexamples]: https://github.com/rtic-rs/rtic/tree/master/rtic/examples

## Running an example

To run the examples with QEMU you will need the `qemu-system-arm` program.
Check [the embedded Rust book] for instructions on how to set up an
embedded development environment that includes QEMU.

[the embedded Rust book]: https://rust-embedded.github.io/book/intro/install.html

To run the examples found in `examples/` locally, cargo needs a supported `target` and
either `--examples` (run all examples) or `--example NAME` to run a specific example.

Assuming dependencies in place, running:

``` console
$ cargo run --target thumbv7m-none-eabi --example locals
```

Yields this output:

``` console
{{#include ../../../rtic/ci/expected/locals.run}}
```

> **NOTE**: You can choose target device by passing a target
> triple to cargo (e.g. `cargo run --example init --target thumbv7m-none-eabi`) or
> configure a default target in `.cargo/config.toml`.
>
> For running the examples, we (typically) use a Cortex M3 emulated in QEMU, so the target is `thumbv7m-none-eabi`.
> Since the M3 architecture is backwards compatible to the M0/M0+ architecture, you may also use the `thumbv6m-none-eabi`, in case you want to inspect generated assembly code for the M0/M0+ architecture.
