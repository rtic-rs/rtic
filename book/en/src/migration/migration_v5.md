# Migrating from v0.5.x to v0.6.0

This section describes how to upgrade from v0.5.x to v0.6.0 of the RTIC framework.

## `Cargo.toml` - version bump

Change the version of `cortex-m-rtic` to `"0.6.0"`.

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

## Move Dispatchers from `extern "C"` to app arguments.

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
    #[init]
    fn init(_: init::Context) -> (init::LateResources, init::Monotonics) {
        rtic::pend(Interrupt::UART0);

        (init::LateResources {}, init::Monotonics())
    }

    // [more code]
}
```

## Resources struct - `#[resources]`

Previously the RTIC resources had to be in in a struct named exactly "Resources":

``` rust
struct Resources {
    // Resources defined in here
}
```

With RTIC v0.6.0 the resources struct is annotated similarly like
`#[task]`, `#[init]`, `#[idle]`: with an attribute `#[resources]`

``` rust
#[resources]
struct Resources {
    // Resources defined in here
}
```

In fact, the name of the struct is now up to the developer:

``` rust
#[resources]
struct Whateveryouwant {
    // Resources defined in here
}
```

would work equally well.

## Spawn/schedule from anywhere

With the new "spawn/schedule from anywhere", old code such as:



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
    foo::schedule(/* ... */).unwrap();
}
```

Note that the attributes `spawn` and `schedule` are no longer needed.

## Symmetric locks

Now RTIC utilizes symmetric locks, this means that the `lock` method need to be used for all resource access. In old code one could do the following as the high priority task has exclusive access to the resource:

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
#[task(priority = 2, resources = [r])]
fn foo(cx: foo::Context) {
    cx.resources.r.lock(|r| r = /* ... */);
}

#[task(resources = [r])]
fn bar(cx: bar::Context) {
    cx.resources.r.lock(|r| r = /* ... */);
}
```

Note that the performance does not change thanks to LLVM's optimizations which optimizes away unnecessary locks.

---

## Additions

### Extern tasks

Both software and hardware tasks can now be defined external to the `mod app`. Previously this was possible only by implementing a trampoline calling out the task implementation.

See examples `examples/extern_binds.rs` and `examples/extern_spawn.rs`.

