# RTIC by example

This part of the book introduces the RTIC framework to new users by walking them through examples of increasing complexity.

All examples in this part of the book are part of the
[RTIC repository][repoexamples], found in the `examples` directory.
The examples are runnable on QEMU (emulating a Cortex M3 target),
thus no special hardware required to follow along.

[repoexamples]: https://github.com/rtic-rs/rtic/tree/master/rtic/examples

## Running an example

To run the examples with QEMU you will need the `qemu-system-arm` program.
Check [the embedded Rust book] for instructions on how to set up an
embedded development environment that includes QEMU.

[the embedded Rust book]: https://rust-embedded.github.io/book/intro/install.html

To run the examples found in `examples/` locally using QEMU:

```
cargo xtask qemu
```

This runs all of the examples against the default `thumbv7m-none-eabi` device `lm3s6965`.

To limit which examples are being run, use the flag `--example <example name>`, the name being the filename of the example.

Assuming dependencies in place, running:

```console
$ cargo xtask qemu --example locals
```

Yields this output:

```console
   Finished dev [unoptimized + debuginfo] target(s) in 0.07s
    Running `target/debug/xtask qemu --example locals`
INFO  xtask > Testing for platform: Lm3s6965, backend: Thumbv7
INFO  xtask::run > 👟 Build example locals (thumbv7m-none-eabi, release, "test-critical-section,thumbv7-backend", in examples/lm3s6965)
INFO  xtask::run > ✅ Success.
INFO  xtask::run > 👟 Run example locals in QEMU (thumbv7m-none-eabi, release, "test-critical-section,thumbv7-backend", in examples/lm3s6965)
INFO  xtask::run > ✅ Success.
INFO  xtask::results > ✅ Success: Build example locals (thumbv7m-none-eabi, release, "test-critical-section,thumbv7-backend", in examples/lm3s6965)
INFO  xtask::results > ✅ Success: Run example locals in QEMU (thumbv7m-none-eabi, release, "test-critical-section,thumbv7-backend", in examples/lm3s6965)
INFO  xtask::results > 🚀🚀🚀 All tasks succeeded 🚀🚀🚀
```

It is great that examples are passing and this is part of the RTIC CI setup too, but for the purposes of this book we must add the `--verbose` flag, or `-v` for short to see the actual program output:

```console
❯ cargo xtask qemu --verbose --example locals
    Finished dev [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/debug/xtask qemu --example locals --verbose`
 DEBUG xtask > Stderr of child processes is inherited: false
 DEBUG xtask > Partial features: false
 INFO  xtask > Testing for platform: Lm3s6965, backend: Thumbv7
 INFO  xtask::run > 👟 Build example locals (thumbv7m-none-eabi, release, "test-critical-section,thumbv7-backend", in examples/lm3s6965)
 INFO  xtask::run > ✅ Success.
 INFO  xtask::run > 👟 Run example locals in QEMU (thumbv7m-none-eabi, release, "test-critical-section,thumbv7-backend", in examples/lm3s6965)
 INFO  xtask::run > ✅ Success.
 INFO  xtask::results > ✅ Success: Build example locals (thumbv7m-none-eabi, release, "test-critical-section,thumbv7-backend", in examples/lm3s6965)
    cd examples/lm3s6965 && cargo build --target thumbv7m-none-eabi --features test-critical-section,thumbv7-backend --release --example locals
 DEBUG xtask::results >
cd examples/lm3s6965 && cargo build --target thumbv7m-none-eabi --features test-critical-section,thumbv7-backend --release --example locals
Stderr:
    Finished release [optimized] target(s) in 0.02s
 INFO  xtask::results > ✅ Success: Run example locals in QEMU (thumbv7m-none-eabi, release, "test-critical-section,thumbv7-backend", in examples/lm3s6965)
    cd examples/lm3s6965 && cargo run --target thumbv7m-none-eabi --features test-critical-section,thumbv7-backend --release --example locals
 DEBUG xtask::results >
cd examples/lm3s6965 && cargo run --target thumbv7m-none-eabi --features test-critical-section,thumbv7-backend --release --example locals
Stdout:
bar: local_to_bar = 1
foo: local_to_foo = 1
idle: local_to_idle = 1

Stderr:
    Finished release [optimized] target(s) in 0.02s
     Running `qemu-system-arm -cpu cortex-m3 -machine lm3s6965evb -nographic -semihosting-config enable=on,target=native -kernel target/thumbv7m-none-eabi/release/examples/locals`
Timer with period zero, disabling

 INFO  xtask::results > 🚀🚀🚀 All tasks succeeded 🚀🚀🚀
```

Look for the content following `Stdout:` towards the end ouf the output, the program output should have these lines:

```console
{{#include ../../../ci/expected/lm3s6965/locals.run}}
```

> **NOTE**: 
> For other useful options to `cargo xtask`, see:
> ```
> cargo xtask qemu --help
> ```
> 
> The `--platform` flag allows changing which device examples are run on,
> currently `lm3s6965` is the best supported, work is ongoing to 
> increase support for other devices, including both ARM and RISC-V
