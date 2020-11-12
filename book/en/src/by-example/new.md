# Starting a new project

Now that you have learned about the main features of the RTIC framework you can
try it out on your hardware by following these instructions.

1. Instantiate the [`cortex-m-quickstart`] template.

[`cortex-m-quickstart`]: https://github.com/rust-embedded/cortex-m-quickstart#cortex-m-quickstart

``` console
$ # for example using `cargo-generate`
$ cargo generate \
    --git https://github.com/rust-embedded/cortex-m-quickstart \
    --name app

$ # follow the rest of the instructions
```

2. Add a peripheral access crate (PAC) that was generated using [`svd2rust`]
   **v0.14.x**, or a board support crate that depends on one such PAC as a
   dependency. Make sure that the `rt` feature of the crate is enabled.

[`svd2rust`]: https://crates.io/crates/svd2rust

In this example, I'll use the [`lm3s6965`] device crate. This device crate
doesn't have an `rt` Cargo feature; that feature is always enabled.

[`lm3s6965`]: https://crates.io/crates/lm3s6965

This device crate provides a linker script with the memory layout of the target
device so `memory.x` and `build.rs` need to be removed.

``` console
$ cargo add lm3s6965 --vers 0.1.3

$ rm memory.x build.rs
```

3. Add the `cortex-m-rtic` crate as a dependency.

``` console
$ cargo add cortex-m-rtic --allow-prerelease
```

4. Write your RTIC application.

Here I'll use the `init` example from the `cortex-m-rtic` crate.

The examples are found in the `examples` folder, and the contents
of `init.rs` is shown here:

``` console
{{#include ../../../../examples/init.rs}}
```

The `init` example uses the `lm3s6965` device. Remember to adjust the `device`
argument in the app macro attribute to match the path of your PAC crate, if
different, and add peripherals or other arguments if needed. Although aliases
can be used, this needs to be a full path (from the crate root). For many
devices, it is common for the HAL implementation crate (aliased as `hal`) or
Board Support crate to re-export the PAC as `pac`, leading to a pattern similar
to the below:

```rust
use abcd123_hal as hal;
//...

#[rtic::app(device = crate::hal::pac, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
mod app { /*...*/ }
```

The `init` example also depends on the `panic-semihosting` crate:

``` console
$ cargo add panic-semihosting
```

5. Build it, flash it and run it.

``` console
$ # NOTE: I have uncommented the `runner` option in `.cargo/config`
$ cargo run
{{#include ../../../../ci/expected/init.run}}
```
