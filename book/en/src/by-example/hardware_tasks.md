# Hardware tasks

At its core RTIC is using the hardware interrupt controller ([ARM NVIC on cortex-m][NVIC])
to perform scheduling and executing tasks, and all tasks except `#[init]` and `#[idle]`
run as interrupt handlers.
This also means that you can manually bind tasks to interrupt handlers.

To bind an interrupt use the `#[task]` attribute argument `binds = InterruptName`.
This task becomes the interrupt handler for this hardware interrupt vector.

All tasks bound to an explicit interrupt are *hardware tasks* since they
start execution in reaction to a hardware event.

Specifying a non-existing interrupt name will cause a compilation error. The interrupt names
are commonly defined by [PAC or HAL][pacorhal] crates.

Any available interrupt vector should work, but different hardware might have
added special properties to select interrupt priority levels, such as the
[nRF “softdevice”](https://github.com/rtic-rs/cortex-m-rtic/issues/434).

Beware of re-purposing interrupt vectors used internally by hardware features,
RTIC is unaware of such hardware specific details.

[pacorhal]: https://docs.rust-embedded.org/book/start/registers.html
[NVIC]: https://developer.arm.com/documentation/100166/0001/Nested-Vectored-Interrupt-Controller/NVIC-functional-description/NVIC-interrupts

The example below demonstrates the use of the `#[task(binds = InterruptName)]` attribute to declare a
hardware task bound to an interrupt handler.

``` rust
{{#include ../../../../examples/hardware.rs}}
```

``` console
$ cargo run --target thumbv7m-none-eabi --example hardware
{{#include ../../../../ci/expected/hardware.run}}
```
