# The background task `#[idle]`

A function marked with the `idle` attribute can optionally appear in the
module. This function is used as the special *idle task* and must have
signature `fn(idle::Context) -> !`.

When present, the runtime will execute the `idle` task after `init`. Unlike
`init`, `idle` will run *with interrupts enabled* and it's not allowed to return
so it must run forever.

When no `idle` function is declared, the runtime sets the [SLEEPONEXIT] bit and
then sends the microcontroller to sleep after running `init`.

[SLEEPONEXIT]: https://developer.arm.com/docs/100737/0100/power-management/sleep-mode/sleep-on-exit-bit

Like in `init`, locally declared resources will have `'static` lifetimes that are safe to access.

The example below shows that `idle` runs after `init`.

``` rust
{{#include ../../../../examples/idle.rs}}
```

``` console
$ cargo run --target thumbv7m-none-eabi --example idle
{{#include ../../../../ci/expected/idle.run}}
```
