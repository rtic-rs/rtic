# App initialization and the `#[init]` task

An RTIC application requires an `init` task setting up the system. The corresponding `init` function must have the
signature `fn(init::Context) -> (Shared, Local, init::Monotonics)`, where `Shared` and `Local` are the resource
structures defined by the user.

The `init` task executes after system reset (after the optionally defined `pre-init` and internal RTIC
initialization). The `init` task runs *with interrupts disabled* and has exclusive access to Cortex-M (the
`bare_metal::CriticalSection` token is available as `cs`) while device specific peripherals are available through
the `core` and `device` fields of `init::Context`.

## Example

The example below shows the types of the `core`, `device` and `cs` fields, and showcases the use of a `local`
variable with `'static` lifetime.
Such variables can be delegated from the `init` task to other tasks of the RTIC application.

The `device` field is available when the `peripherals` argument is set to the default value `true`.
In the rare case you want to implement an ultra-slim application you can explicitly set `peripherals` to `false`.

``` rust
{{#include ../../../../examples/init.rs}}
```

Running the example will print `init` to the console and then exit the QEMU process.

``` console
$ cargo run --target thumbv7m-none-eabi --example init
{{#include ../../../../ci/expected/init.run}}
```

> **NOTE**: You can choose target device by passing a target
> triple to cargo (e.g. `cargo run --example init --target thumbv7m-none-eabi`) or
> configure a default target in `.cargo/config.toml`.
>
> For running the examples, we use a Cortex M3 emulated in QEMU, so the target is `thumbv7m-none-eabi`.
