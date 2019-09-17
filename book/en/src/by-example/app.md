# The `app` attribute

This is the smallest possible RTFM application:

``` rust
{{#include ../../../../examples/smallest.rs}}
```

All RTFM applications use the [`app`] attribute (`#[app(..)]`). This attribute
must be applied to a `const` item that contains items. The `app` attribute has
a mandatory `device` argument that takes a *path* as a value. This path must
point to a *peripheral access crate* (PAC) generated using [`svd2rust`]
**v0.14.x** or newer. The `app` attribute will expand into a suitable entry
point so it's not required to use the [`cortex_m_rt::entry`] attribute.

[`app`]: ../../../api/cortex_m_rtfm_macros/attr.app.html
[`svd2rust`]: https://crates.io/crates/svd2rust
[`cortex_m_rt::entry`]: ../../../api/cortex_m_rt_macros/attr.entry.html

> **ASIDE**: Some of you may be wondering why we are using a `const` item as a
> module and not a proper `mod` item. The reason is that using attributes on
> modules requires a feature gate, which requires a nightly toolchain. To make
> RTFM work on stable we use the `const` item instead. When more parts of macros
> 1.2 are stabilized we'll move from a `const` item to a `mod` item and
> eventually to a crate level attribute (`#![app]`).

## `init`

Within the pseudo-module the `app` attribute expects to find an initialization
function marked with the `init` attribute. This function must have signature
`fn(init::Context) [-> init::LateResources]` (the return type is not always
required).

This initialization function will be the first part of the application to run.
The `init` function will run *with interrupts disabled* and has exclusive access
to Cortex-M and, optionally, device specific peripherals through the `core` and
`device` fields of `init::Context`.

`static mut` variables declared at the beginning of `init` will be transformed
into `&'static mut` references that are safe to access.

[`rtfm::Peripherals`]: ../../api/rtfm/struct.Peripherals.html

The example below shows the types of the `core` and `device` fields and
showcases safe access to a `static mut` variable. The `device` field is only
available when the `peripherals` argument is set to `true` (it defaults to
`false`).

``` rust
{{#include ../../../../examples/init.rs}}
```

Running the example will print `init` to the console and then exit the QEMU
process.

```  console
$ cargo run --example init
{{#include ../../../../ci/expected/init.run}}```

## `idle`

A function marked with the `idle` attribute can optionally appear in the
pseudo-module. This function is used as the special *idle task* and must have
signature `fn(idle::Context) - > !`.

When present, the runtime will execute the `idle` task after `init`. Unlike
`init`, `idle` will run *with interrupts enabled* and it's not allowed to return
so it must run forever.

When no `idle` function is declared, the runtime sets the [SLEEPONEXIT] bit and
then sends the microcontroller to sleep after running `init`.

[SLEEPONEXIT]: https://developer.arm.com/products/architecture/cpu-architecture/m-profile/docs/100737/0100/power-management/sleep-mode/sleep-on-exit-bit

Like in `init`, `static mut` variables will be transformed into `&'static mut`
references that are safe to access.

The example below shows that `idle` runs after `init`.

``` rust
{{#include ../../../../examples/idle.rs}}
```

``` console
$ cargo run --example idle
{{#include ../../../../ci/expected/idle.run}}```

## Hardware tasks

To declare interrupt handlers the framework provides a `#[task]` attribute that
can be attached to functions. This attribute takes a `binds` argument whose
value is the name of the interrupt to which the handler will be bound to; the
function adornated with this attribute becomes the interrupt handler. Within the
framework these type of tasks are referred to as *hardware* tasks, because they
start executing in reaction to a hardware event.

The example below demonstrates the use of the `#[task]` attribute to declare an
interrupt handler. Like in the case of `#[init]` and `#[idle]` local `static
mut` variables are safe to use within a hardware task.

``` rust
{{#include ../../../../examples/hardware.rs}}
```

``` console
$ cargo run --example interrupt
{{#include ../../../../ci/expected/hardware.run}}```

So far all the RTFM applications we have seen look no different that the
applications one can write using only the `cortex-m-rt` crate. From this point
we start introducing features unique to RTFM.

## Priorities

The static priority of each handler can be declared in the `task` attribute
using the `priority` argument. Tasks can have priorities in the range `1..=(1 <<
NVIC_PRIO_BITS)` where `NVIC_PRIO_BITS` is a constant defined in the `device`
crate. When the `priority` argument is omitted the priority is assumed to be
`1`. The `idle` task has a non-configurable static priority of `0`, the lowest
priority.

When several tasks are ready to be executed the one with *highest* static
priority will be executed first. Task prioritization can be observed in the
following scenario: an interrupt signal arrives during the execution of a low
priority task; the signal puts the higher priority task in the pending state.
The difference in priority results in the higher priority task preempting the
lower priority one: the execution of the lower priority task is suspended and
the higher priority task is executed to completion. Once the higher priority
task has terminated the lower priority task is resumed.

The following example showcases the priority based scheduling of tasks.

``` rust
{{#include ../../../../examples/preempt.rs}}
```

``` console
$ cargo run --example interrupt
{{#include ../../../../ci/expected/preempt.run}}```

Note that the task `gpiob` does *not* preempt task `gpioc` because its priority
is the *same* as `gpioc`'s. However, once `gpioc` terminates the execution of
task `gpiob` is prioritized over `gpioa`'s due to its higher priority. `gpioa`
is resumed only after `gpiob` terminates.

One more note about priorities: choosing a priority higher than what the device
supports (that is `1 << NVIC_PRIO_BITS`) will result in a compile error. Due to
limitations in the language the error message is currently far from helpful: it
will say something along the lines of "evaluation of constant value failed" and
the span of the error will *not* point out to the problematic interrupt value --
we are sorry about this!
