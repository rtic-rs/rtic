# Hardware tasks

In it's core RTIC is based on using the interrupt controller in the hardware to do scheduling and
run tasks, as all tasks in the framework are run as interrupt handlers (except `#[init]` and
`#[idle]`). This also means that you can directly bind tasks to interrupt handlers.

To declare interrupt handlers the  `#[task]` attribute takes a `binds = InterruptName` argument whose
value is the name of the interrupt to which the handler will be bound to; the
function used with this attribute becomes the interrupt handler. Within the
framework these type of tasks are referred to as *hardware* tasks, because they
start executing in reaction to a hardware event.

Providing an interrupt name that does not exist will cause a compile error to help with accidental
errors.

The example below demonstrates the use of the `#[task]` attribute to declare an
interrupt handler.

``` rust
{{#include ../../../../examples/hardware.rs}}
```

``` console
$ cargo run --target thumbv7m-none-eabi --example hardware
{{#include ../../../../ci/expected/hardware.run}}
```

