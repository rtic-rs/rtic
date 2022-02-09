# Migrating from v0.5.x to v1.0.0

This section describes how to upgrade from v0.5.x to v1.0.0 of the RTIC framework.

## `Cargo.toml` - version bump

Change the version of `cortex-m-rtic` to `"1.0.0"`.

## `mod` instead of `const`

With the support of attributes on modules the `const APP` workaround is not needed.

Change

``` rust
#[rtic::app(/* .. */)]
const APP: () = {
  [code here]
};
```

into

``` rust
#[rtic::app(/* .. */)]
mod app {
  [code here]
}
```

Now that a regular Rust module is used it means it is possible to have custom
user code within that module.
Additionally, it means that `use`-statements for resources used in user
code must be moved inside `mod app`, or be referred to with `super`. For
example, change:

```rust
use some_crate::some_func;

#[rtic::app(/* .. */)]
const APP: () = {
    fn func() {
        some_crate::some_func();
    }
};
```

into

```rust
#[rtic::app(/* .. */)]
mod app {
    use some_crate::some_func;

    fn func() {
        some_crate::some_func();
    }
}
```

or

```rust
use some_crate::some_func;

#[rtic::app(/* .. */)]
mod app {
    fn func() {
        super::some_crate::some_func();
    }
}
```

## Move Dispatchers from `extern "C"` to app arguments

Change

``` rust
#[rtic::app(/* .. */)]
const APP: () = {
    [code here]

    // RTIC requires that unused interrupts are declared in an extern block when
    // using software tasks; these free interrupts will be used to dispatch the
    // software tasks.
    extern "C" {
        fn SSI0();
        fn QEI0();
    }
};
```

into

``` rust
#[rtic::app(/* .. */, dispatchers = [SSI0, QEI0])]
mod app {
  [code here]
}
```

This works also for ram functions, see examples/ramfunc.rs


## Resources structs - `#[shared]`, `#[local]`

Previously the RTIC resources had to be in in a struct named exactly "Resources":

``` rust
struct Resources {
    // Resources defined in here
}
```

With RTIC v1.0.0 the resources structs are annotated similarly like
`#[task]`, `#[init]`, `#[idle]`: with the attributes `#[shared]` and `#[local]`

``` rust
#[shared]
struct MySharedResources {
    // Resources shared between tasks are defined here
}

#[local]
struct MyLocalResources {
    // Resources defined here cannot be shared between tasks; each one is local to a single task
}
```

These structs can be freely named by the developer.

## `shared` and `local` arguments in `#[task]`s

In v1.0.0 resources are split between `shared` resources and `local` resources.
`#[task]`, `#[init]` and `#[idle]` no longer have a `resources` argument; they must now use the `shared` and `local` arguments.

In v0.5.x:

``` rust
struct Resources {
    local_to_b: i64,
    shared_by_a_and_b: i64,
}

#[task(resources = [shared_by_a_and_b])]
fn a(_: a::Context) {}

#[task(resources = [shared_by_a_and_b, local_to_b])]
fn b(_: b::Context) {}
```

In v1.0.0:

``` rust
#[shared]
struct Shared {
    shared_by_a_and_b: i64,
}

#[local]
struct Local {
    local_to_b: i64,
}

#[task(shared = [shared_by_a_and_b])]
fn a(_: a::Context) {}

#[task(shared = [shared_by_a_and_b], local = [local_to_b])]
fn b(_: b::Context) {}
```

## Symmetric locks

Now RTIC utilizes symmetric locks, this means that the `lock` method need
to be used for all `shared` resource access.
In old code one could do the following as the high priority
task has exclusive access to the resource:

``` rust
#[task(priority = 2, resources = [r])]
fn foo(cx: foo::Context) {
    cx.resources.r = /* ... */;
}

#[task(resources = [r])]
fn bar(cx: bar::Context) {
    cx.resources.r.lock(|r| r = /* ... */);
}
```

And with symmetric locks one needs to use locks in both tasks:

``` rust
#[task(priority = 2, shared = [r])]
fn foo(cx: foo::Context) {
    cx.shared.r.lock(|r| r = /* ... */);
}

#[task(shared = [r])]
fn bar(cx: bar::Context) {
    cx.shared.r.lock(|r| r = /* ... */);
}
```

Note that the performance does not change thanks to LLVM's optimizations which optimizes away unnecessary locks.

## Lock-free resource access

In RTIC 0.5 resources shared by tasks running at the same priority could be accessed *without* the `lock` API.
This is still possible in 1.0: the `#[shared]` resource must be annotated with the field-level `#[lock_free]` attribute.

v0.5 code:

``` rust
struct Resources {
    counter: u64,
}

#[task(resources = [counter])]
fn a(cx: a::Context) {
    *cx.resources.counter += 1;
}

#[task(resources = [counter])]
fn b(cx: b::Context) {
    *cx.resources.counter += 1;
}
```

v1.0 code:

``` rust
#[shared]
struct Shared {
    #[lock_free]
    counter: u64,
}

#[task(shared = [counter])]
fn a(cx: a::Context) {
    *cx.shared.counter += 1;
}

#[task(shared = [counter])]
fn b(cx: b::Context) {
    *cx.shared.counter += 1;
}
```

## no `static mut` transform

`static mut` variables are no longer transformed to safe `&'static mut` references.
Instead of that syntax, use the `local` argument in `#[init]`.

v0.5.x code:

``` rust
#[init]
fn init(_: init::Context) {
    static mut BUFFER: [u8; 1024] = [0; 1024];
    let buffer: &'static mut [u8; 1024] = BUFFER;
}
```

v1.0.0 code:

``` rust
#[init(local = [
    buffer: [u8; 1024] = [0; 1024]
//   type ^^^^^^^^^^^^   ^^^^^^^^^ initial value
])]
fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
    let buffer: &'static mut [u8; 1024] = cx.local.buffer;

    (Shared {}, Local {}, init::Monotonics())
}
```

## Init always returns late resources

In order to make the API more symmetric the #[init]-task always returns a late resource.

From this:

``` rust
#[rtic::app(device = lm3s6965)]
const APP: () = {
    #[init]
    fn init(_: init::Context) {
        rtic::pend(Interrupt::UART0);
    }

    // [more code]
};
```

to this:

``` rust
#[rtic::app(device = lm3s6965)]
mod app {
    #[shared]
    struct MySharedResources {}

    #[local]
    struct MyLocalResources {}

    #[init]
    fn init(_: init::Context) -> (MySharedResources, MyLocalResources, init::Monotonics) {
        rtic::pend(Interrupt::UART0);

        (MySharedResources, MyLocalResources, init::Monotonics())
    }

    // [more code]
}
```

## Spawn from anywhere

With the new spawn/spawn_after/spawn_at interface,
old code requiring the context `cx` for spawning such as:

``` rust
#[task(spawn = [bar])]
fn foo(cx: foo::Context) {
    cx.spawn.bar().unwrap();
}

#[task(schedule = [bar])]
fn bar(cx: bar::Context) {
    cx.schedule.foo(/* ... */).unwrap();
}
```

Will now be written as:

``` rust
#[task]
fn foo(_c: foo::Context) {
    bar::spawn().unwrap();
}

#[task]
fn bar(_c: bar::Context) {
    // Takes a Duration, relative to “now”
    let spawn_handle = foo::spawn_after(/* ... */);
}

#[task]
fn bar(_c: bar::Context) {
    // Takes an Instant
    let spawn_handle = foo::spawn_at(/* ... */);
}
```

Thus the requirement of having access to the context is dropped.

Note that the attributes `spawn`/`schedule` in the task definition are no longer needed.

---

## Additions

### Extern tasks

Both software and hardware tasks can now be defined external to the `mod app`.
Previously this was possible only by implementing a trampoline calling out the task implementation.

See examples `examples/extern_binds.rs` and `examples/extern_spawn.rs`.
