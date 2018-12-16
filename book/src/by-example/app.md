# The `app` attribute

This is the smallest possible RTFM application:

``` rust
{{#include ../../../examples/smallest.rs}}
```

All RTFM applications use the [`app`] attribute (`#[app(..)]`). This attribute
must be applied to a `const` item that contains items. The `app` attribute has
a mandatory `device` argument that takes a *path* as a value. This path must
point to a *peripheral access crate* (PAC) generated using [`svd2rust`]
**v0.14.x**. The `app` attribute will expand into a suitable entry point so it's
not required to use the [`cortex_m_rt::entry`] attribute.

[`app`]: ../../api/cortex_m_rtfm_macros/attr.app.html
[`svd2rust`]: https://crates.io/crates/svd2rust
[`cortex_m_rt::entry`]: ../../api/cortex_m_rt_macros/attr.entry.html

> **ASIDE**: Some of you may be wondering why we are using a `const` item as a
> module and not a proper `mod` item. The reason is that using attributes on
> modules requires a feature gate, which requires a nightly toolchain. To make
> RTFM work on stable we use the `const` item instead. When more parts of macros
> 1.2 are stabilized we'll move from a `const` item to a `mod` item and
> eventually to a crate level attribute (`#![app]`).

## `init`

Within the pseudo-module the `app` attribute expects to find an initialization
function marked with the `init` attribute. This function must have signature
`[unsafe] fn()`.

This initialization function will be the first part of the application to run.
The `init` function will run *with interrupts disabled* and has exclusive access
to Cortex-M and device specific peripherals through the `core` and `device`
variables, which are injected in the scope of `init` by the `app` attribute. Not
all Cortex-M peripherals are available in `core` because the RTFM runtime takes
ownership of some of them -- for more details see the [`rtfm::Peripherals`]
struct.

`static mut` variables declared at the beginning of `init` will be transformed
into `&'static mut` references that are safe to access.

[`rtfm::Peripherals`]: ../../api/rtfm/struct.Peripherals.html

The example below shows the types of the `core` and `device` variables and
showcases safe access to a `static mut` variable.

``` rust
{{#include ../../../examples/init.rs}}
```

Running the example will print `init` to the console and then exit the QEMU
process.

```  console
$ cargo run --example init
{{#include ../../../ci/expected/init.run}}```

## `idle`

A function marked with the `idle` attribute can optionally appear in the
pseudo-module. This function is used as the special *idle task* and must have
signature `[unsafe] fn() - > !`.

When present, the runtime will execute the `idle` task after `init`. Unlike
`init`, `idle` will run *with interrupts enabled* and it's not allowed to return
so it runs forever.

When no `idle` function is declared, the runtime sets the [SLEEPONEXIT] bit and
then sends the microcontroller to sleep after running `init`.

[SLEEPONEXIT]: https://developer.arm.com/products/architecture/cpu-architecture/m-profile/docs/100737/0100/power-management/sleep-mode/sleep-on-exit-bit

Like in `init`, `static mut` variables will be transformed into `&'static mut`
references that are safe to access.

The example below shows that `idle` runs after `init`.

``` rust
{{#include ../../../examples/idle.rs}}
```

``` console
$ cargo run --example idle
{{#include ../../../ci/expected/idle.run}}```

## `interrupt` / `exception`

Just like you would do with the `cortex-m-rt` crate you can use the `interrupt`
and `exception` attributes within the `app` pseudo-module to declare interrupt
and exception handlers. In RTFM, we refer to interrupt and exception handlers as
*hardware* tasks.

``` rust
{{#include ../../../examples/interrupt.rs}}
```

``` console
$ cargo run --example interrupt
{{#include ../../../ci/expected/interrupt.run}}```

So far all the RTFM applications we have seen look no different that the
applications one can write using only the `cortex-m-rt` crate. In the next
section we start introducing features unique to RTFM.
