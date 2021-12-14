# The background task `#[idle]`

A function marked with the `idle` attribute can optionally appear in the
module. This becomes the special *idle task* and must have signature
`fn(idle::Context) -> !`.

When present, the runtime will execute the `idle` task after `init`. Unlike
`init`, `idle` will run *with interrupts enabled* and must never return,
as the `-> !` function signature indicates.
[The Rust type `!` means “never”][nevertype].

[nevertype]: https://doc.rust-lang.org/core/primitive.never.html

Like in `init`, locally declared resources will have `'static` lifetimes that
are safe to access.

The example below shows that `idle` runs after `init`.

``` rust
{{#include ../../../../examples/idle.rs}}
```

``` console
$ cargo run --target thumbv7m-none-eabi --example idle
{{#include ../../../../ci/expected/idle.run}}
```

By default, the RTIC `idle` task does not try to optimize for any specific targets.

A common useful optimization is to enable the [SLEEPONEXIT] and allow the MCU
to enter sleep when reaching `idle`.

>**Caution** some hardware unless configured disables the debug unit during sleep mode.
>
>Consult your hardware specific documentation as this is outside the scope of RTIC.

The following example shows how to enable sleep by setting the
[`SLEEPONEXIT`][SLEEPONEXIT] and providing a custom `idle` task replacing the
default [`nop()`][NOP] with [`wfi()`][WFI].

[SLEEPONEXIT]: https://developer.arm.com/docs/100737/0100/power-management/sleep-mode/sleep-on-exit-bit
[WFI]: https://developer.arm.com/documentation/dui0662/b/The-Cortex-M0--Instruction-Set/Miscellaneous-instructions/WFI
[NOP]: https://developer.arm.com/documentation/dui0662/b/The-Cortex-M0--Instruction-Set/Miscellaneous-instructions/NOP

``` rust
{{#include ../../../../examples/idle-wfi.rs}}
```

``` console
$ cargo run --target thumbv7m-none-eabi --example idle-wfi
{{#include ../../../../ci/expected/idle-wfi.run}}
```
