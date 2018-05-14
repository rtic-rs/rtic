# The scheduler

The RTFM framework includes a priority based scheduler. In the Cortex-M implementation of RTFM the
[NVIC][] (Nested Vector Interrupt Controller), a Cortex-M core peripheral, does the actual task
scheduling -- this greatly reduces the bookkeeping that needs to be done in software.

[NVIC]: https://developer.arm.com/docs/ddi0337/e/nested-vectored-interrupt-controller

All tasks map one way or another to an interrupt. This lets the NVIC schedule tasks as if they were
interrupts. The NVIC dispatches interrupt handlers according to their priorities; this gives up
priority based scheduling of tasks for free.

The NVIC contains a interrupt priority registers (IPR) where the *static* priority of an interrupt
can be set. The priorities assigned to tasks by the user are programmed into these registers after
`init` ends and before `idle` starts, while the interrupts are disabled.

The IPR registers store priorities in a different way than the user specifies them so a conversion
is needed. To distinguish these two we refer to the IPR format as *hardware* priority level, and we
refer to the priority entered in `app!` as the *logical* priority level.

In hardware priority levels a bigger number indicates *lower* urgency and vice versa. Plus, Cortex-M
devices only support a certain number of priority bits: for example 4 bits equates 16 different
priority levels. These priority bits correspond to the higher bits of each 8-bit IPR register.

Different devices support different number of priority bits so this needs to be accounted for when
converting from a logical priority level to a hardware priority level. This is what the conversion
routine looks like:

``` rust
// number of priority bits (device specific)
const NVIC_PRIO_BITS: u8 = 4;

fn logical2hardware(prio: u8) -> u8 {
    ((1 << NVIC_PRIO_BITS) - prio) << (8 - NVIC_PRIO_BITS)
}
```

The RTFM runtime needs to know `NVIC_PRIO_BITS` for the target device to properly configure the
priority of each task. Currently the `app!` macro expects the `device` crate to contain this
information as a `u8` constant at `$device::NVIC_PRIO_BITS`.
