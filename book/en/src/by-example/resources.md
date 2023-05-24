# Resource usage

The RTIC framework manages shared and task local resources allowing persistent data storage and safe accesses without the use of `unsafe` code.

RTIC resources are visible only to functions declared within the `#[app]` module and the framework gives the user complete control (on a per-task basis) over resource accessibility.

Declaration of system-wide resources is done by annotating **two** `struct`s within the `#[app]` module with the attribute `#[local]` and `#[shared]`. Each field in these structures corresponds to a different resource (identified by field name). The difference between these two sets of resources will be covered below.

Each task must declare the resources it intends to access in its corresponding metadata attribute using the `local` and `shared` arguments. Each argument takes a list of resource identifiers. The listed resources are made available to the context under the `local` and `shared` fields of the `Context` structure.

The `init` task returns the initial values for the system-wide (`#[shared]` and `#[local]`) resources.
 
<!-- and the set of initialized timers used by the application. The monotonic timers will be
further discussed in [Monotonic & `spawn_{at/after}`](./monotonic.md). -->

## `#[local]` resources

`#[local]` resources are locally accessible to a specific task, meaning that only that task can access the resource and does so without locks or critical sections. This allows for the resources, commonly drivers or large objects, to be initialized in `#[init]` and then be passed to a specific task.

Thus, a task `#[local]` resource can only be accessed by one singular task. Attempting to assign the same `#[local]` resource to more than one task is a compile-time error.

Types of `#[local]` resources must implement a [`Send`] trait as they are being sent from `init` to a target task, crossing a thread boundary.

[`Send`]: https://doc.rust-lang.org/stable/core/marker/trait.Send.html

The example application shown below contains three tasks `foo`, `bar` and `idle`, each having access to its own `#[local]` resource.

``` rust,noplayground
{{#include ../../../../rtic/examples/locals.rs}}
```

Running the example:

``` console
$ cargo run --target thumbv7m-none-eabi --example locals
```

``` console
{{#include ../../../../rtic/ci/expected/locals.run}}
```

Local resources in `#[init]` and `#[idle]` have `'static` lifetimes. This is safe since both tasks are not re-entrant.

### Task local initialized resources

Local resources can also be specified directly in the resource claim like so: `#[task(local = [my_var: TYPE = INITIAL_VALUE, ...])]`; this allows for creating locals which do no need to be initialized in `#[init]`.

Types of `#[task(local = [..])]` resources have to be neither [`Send`] nor [`Sync`] as they are not crossing any thread boundary.

[`Sync`]: https://doc.rust-lang.org/stable/core/marker/trait.Sync.html

In the example below the different uses and lifetimes are shown:

``` rust,noplayground
{{#include ../../../../rtic/examples/declared_locals.rs}}
```

You can run the application, but as the example is designed merely to showcase the lifetime properties there is no output (it suffices to build the application).

``` console
$ cargo build --target thumbv7m-none-eabi --example declared_locals
```
<!-- {{#include ../../../../rtic/ci/expected/declared_locals.run}} -->

## `#[shared]` resources and `lock`

Critical sections are required to access `#[shared]` resources in a data race-free manner and to achieve this the `shared` field of the passed `Context` implements the [`Mutex`] trait for each shared resource accessible to the task. This trait has only one method, [`lock`], which runs its closure argument in a critical section.

[`Mutex`]: ../../../api/rtic/trait.Mutex.html
[`lock`]: ../../../api/rtic/trait.Mutex.html#method.lock

The critical section created by the `lock` API is based on dynamic priorities: it temporarily raises the dynamic priority of the context to a *ceiling* priority that prevents other tasks from preempting the critical section. This synchronization protocol is known as the [Immediate Ceiling Priority Protocol (ICPP)][icpp], and complies with [Stack Resource Policy (SRP)][srp] based scheduling of RTIC.

[icpp]: https://en.wikipedia.org/wiki/Priority_ceiling_protocol
[srp]: https://en.wikipedia.org/wiki/Stack_Resource_Policy

In the example below we have three interrupt handlers with priorities ranging from one to three. The two handlers with the lower priorities contend for a `shared` resource and need to succeed in locking the resource in order to access its data. The highest priority handler, which does not access the `shared` resource, is free to preempt a critical section created by the lowest priority handler.

``` rust,noplayground
{{#include ../../../../rtic/examples/lock.rs}}
```

``` console
$ cargo run --target thumbv7m-none-eabi --example lock
```

``` console
{{#include ../../../../rtic/ci/expected/lock.run}}
```

Types of `#[shared]` resources have to be [`Send`].

## Multi-lock

As an extension to `lock`, and to reduce rightward drift, locks can be taken as tuples. The following examples show this in use:

``` rust,noplayground
{{#include ../../../../rtic/examples/multilock.rs}}
```

``` console
$ cargo run --target thumbv7m-none-eabi --example multilock
```

``` console
{{#include ../../../../rtic/ci/expected/multilock.run}}
```

## Only shared (`&-`) access

By default, the framework assumes that all tasks require exclusive mutable access (`&mut-`) to resources, but it is possible to specify that a task only requires shared access (`&-`) to a resource using the `&resource_name` syntax in the `shared` list.

The advantage of specifying shared access (`&-`) to a resource is that no locks are required to access the resource even if the resource is contended by more than one task running at different priorities. The downside is that the task only gets a shared reference (`&-`) to the resource, limiting the operations it can perform on it, but where a shared reference is enough this approach reduces the number of required locks. In addition to simple immutable data, this shared access can be useful where the resource type safely implements interior mutability, with appropriate locking or atomic operations of its own.

Note that in this release of RTIC it is not possible to request both exclusive access (`&mut-`) and shared access (`&-`) to the *same* resource from different tasks. Attempting to do so will result in a compile error.

In the example below a key (e.g. a cryptographic key) is loaded (or created) at runtime (returned by `init`) and then used from two tasks that run at different priorities without any kind of lock.

``` rust,noplayground
{{#include ../../../../rtic/examples/only-shared-access.rs}}
```

``` console
$ cargo run --target thumbv7m-none-eabi --example only-shared-access
```

``` console
{{#include ../../../../rtic/ci/expected/only-shared-access.run}}
```

## Lock-free access of shared resources

A critical section is *not* required to access a `#[shared]` resource that's only accessed by tasks running at the *same* priority. In this case, you can opt out of the `lock` API by adding the `#[lock_free]` field-level attribute to the resource declaration (see example below). 

<!-- Note that this is merely a convenience to reduce needless resource locking code, because even if the
`lock` API is used, at runtime the framework will **not** produce a critical section due to how
the underlying resource-ceiling preemption works. -->

To adhere to the Rust [aliasing] rule, a resource may be either accessed through multiple immutable references or a singe mutable reference (but not both at the same time). 

[aliasing]: https://doc.rust-lang.org/nomicon/aliasing.html

Using `#[lock_free]` on resources shared by tasks running at different priorities will result in a *compile-time* error -- not using the `lock` API would violate the aforementioned alias rule. Similarly, for each priority there can be only a single *software* task accessing a shared resource (as an `async` task may yield execution to other *software* or *hardware* tasks running at the same priority). However, under this single-task restriction, we make the observation that the resource is in effect no longer `shared` but rather `local`. Thus, using a `#[lock_free]` shared resource will result in a *compile-time* error -- where applicable, use a `#[local]` resource instead.

``` rust,noplayground
{{#include ../../../../rtic/examples/lock-free.rs}}
```

``` console
$ cargo run --target thumbv7m-none-eabi --example lock-free
```

``` console
{{#include ../../../../rtic/ci/expected/lock-free.run}}
```
