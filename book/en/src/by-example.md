# RTIC by example

This part of the book introduces the Real-Time Interrupt-driven Concurrency (RTIC) framework
to new users by walking them through examples of increasing complexity.

All examples in this part of the book are accessible at the
[GitHub repository][repoexamples].
The examples are runnable on QEMU (emulating a Cortex M3 target),
thus no special hardware required to follow along.

[repoexamples]: https://github.com/rtic-rs/cortex-m-rtic/tree/master/examples

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
{{#include ../../../ci/expected/locals.run}}
```
