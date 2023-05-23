# Hardware tasks

At its core RTIC is using a hardware interrupt controller ([ARM NVIC on cortex-m][NVIC]) to schedule and start execution of tasks. All tasks except `pre-init` (a hidden "task"), `#[init]` and `#[idle]` run as interrupt handlers.

To bind a task to an interrupt, use the `#[task]` attribute argument `binds = InterruptName`. This task then becomes the interrupt handler for this hardware interrupt vector.

All tasks bound to an explicit interrupt are called *hardware tasks* since they start execution in reaction to a hardware event.

Specifying a non-existing interrupt name will cause a compilation error. The interrupt names are commonly defined by [PAC or HAL][pacorhal] crates.

Any available interrupt vector should work. Specific devices may bind specific interrupt priorities to specific interrupt vectors outside user code control. See for example the  [nRF “softdevice”](https://github.com/rtic-rs/rtic/issues/434).

Beware of using interrupt vectors that are used internally by hardware features; RTIC is unaware of such hardware specific details.

[pacorhal]: https://docs.rust-embedded.org/book/start/registers.html
[NVIC]: https://developer.arm.com/documentation/100166/0001/Nested-Vectored-Interrupt-Controller/NVIC-functional-description/NVIC-interrupts

## Example

The example below demonstrates the use of the `#[task(binds = InterruptName)]` attribute to declare a hardware task bound to an interrupt handler.

``` rust,noplayground
{{#include ../../../../rtic/examples/hardware.rs}}
```

``` console
$ cargo run --target thumbv7m-none-eabi --example hardware
```

``` console
{{#include ../../../../rtic/ci/expected/hardware.run}}
```
