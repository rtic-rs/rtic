# Migrating from v0.4.x to v0.5.0

This section covers how to upgrade an application written against RTFM v0.4.x to
the version v0.5.0 of the framework.

## `Cargo.toml`

First, the version of the `cortex-m-rtfm` dependency needs to be updated to
`"0.5.0"`. The `timer-queue` feature needs to be removed.


``` toml
[dependencies.cortex-m-rtfm]
# change this
version = "0.4.3"

# into this
version = "0.5.0-beta.1"

# and remove this Cargo feature
features = ["timer-queue"]
#           ^^^^^^^^^^^^^
```

## `Context` argument

All functions inside the `#[rtfm::app]` item need to take as first argument a
`Context` structure. This `Context` type will contain the variables that were
magically injected into the scope of the function by version v0.4.x of the
framework: `resources`, `spawn`, `schedule` -- these variables will become
fields of the `Context` structure. Each function within the `#[rtfm::app]` item
gets a different `Context` type.

``` rust
#[rtfm::app(/* .. */)]
const APP: () = {
    // change this
    #[task(resources = [x], spawn = [a], schedule = [b])]
    fn foo() {
        resources.x.lock(|x| /* .. */);
        spawn.a(message);
        schedule.b(baseline);
    }

    // into this
    #[task(resources = [x], spawn = [a], schedule = [b])]
    fn foo(mut cx: foo::Context) {
        // ^^^^^^^^^^^^^^^^^^^^

        cx.resources.x.lock(|x| /* .. */);
    //  ^^^

        cx.spawn.a(message);
    //  ^^^

        cx.schedule.b(message, baseline);
    //  ^^^
    }

    // change this
    #[init]
    fn init() {
        // ..
    }

    // into this
    #[init]
    fn init(cx: init::Context) {
        //  ^^^^^^^^^^^^^^^^^
        // ..
    }

    // ..
};
```

## Resources

The syntax used to declare resources has been changed from `static mut`
variables to a `struct Resources`.

``` rust
#[rtfm::app(/* .. */)]
const APP: () = {
    // change this
    static mut X: u32 = 0;
    static mut Y: u32 = (); // late resource

    // into this
    struct Resources {
        #[init(0)] // <- initial value
        X: u32, // NOTE: we suggest changing the naming style to `snake_case`

        Y: u32, // late resource
    }

    // ..
};
```

## Device peripherals

If your application was accessing the device peripherals in `#[init]` through
the `device` variable then you'll need to add `peripherals = true` to the
`#[rtfm::app]` attribute to continue to access the device peripherals through
the `device` field of the `init::Context` structure.

Change this:

``` rust
#[rtfm::app(/* .. */)]
const APP: () = {
    #[init]
    fn init() {
        device.SOME_PERIPHERAL.write(something);
    }

    // ..
};
```

Into this:

``` rust
#[rtfm::app(/* .. */, peripherals = true)]
//                    ^^^^^^^^^^^^^^^^^^
const APP: () = {
    #[init]
    fn init(cx: init::Context) {
        //  ^^^^^^^^^^^^^^^^^
        cx.device.SOME_PERIPHERAL.write(something);
    //  ^^^
    }

    // ..
};
```

## `#[interrupt]` and `#[exception]`

The `#[interrupt]` and `#[exception]` attributes have been removed. To declare
hardware tasks in v0.5.x use the `#[task]` attribute with the `binds` argument.

Change this:

``` rust
#[rtfm::app(/* .. */)]
const APP: () = {
    // hardware tasks
    #[exception]
    fn SVCall() { /* .. */ }

    #[interrupt]
    fn UART0() { /* .. */ }

    // software task
    #[task]
    fn foo() { /* .. */ }

    // ..
};
```

Into this:

``` rust
#[rtfm::app(/* .. */)]
const APP: () = {
    #[task(binds = SVCall)]
    //     ^^^^^^^^^^^^^^
    fn svcall(cx: svcall::Context) { /* .. */ }
    // ^^^^^^ we suggest you use a `snake_case` name here

    #[task(binds = UART0)]
    //     ^^^^^^^^^^^^^
    fn uart0(cx: uart0::Context) { /* .. */ }

    #[task]
    fn foo(cx: foo::Context) { /* .. */ }

    // ..
};
```

## `schedule`

The `timer-queue` feature has been removed. To use the `schedule` API one must
first define the monotonic timer the runtime will use using the `monotonic`
argument of the `#[rtfm::app]` attribute. To continue using the cycle counter
(CYCCNT) as the monotonic timer, and match the behavior of version v0.4.x, add
the `monotonic = rtfm::cyccnt::CYCCNT` argument to the `#[rtfm::app]` attribute.

Also, the `Duration` and `Instant` types and the `U32Ext` trait have been moved
into the `rtfm::cyccnt` module. This module is only available on ARMv7-M+
devices.

Change this:

``` rust
use rtfm::{Duration, Instant, U32Ext};

#[rtfm::app(/* .. */)]
const APP: () = {
    #[task(schedule = [b])]
    fn a() {
        // ..
    }
};
```

Into this:

``` rust
use rtfm::cyccnt::{Duration, Instant, U32Ext};
//        ^^^^^^^^

#[rtfm::app(/* .. */, monotonic = rtfm::cyccnt::CYCCNT)]
//                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
const APP: () = {
    #[task(schedule = [b])]
    fn a(cx: a::Context) {
        // ..
    }
};
```
