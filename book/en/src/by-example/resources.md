# Resources

The framework provides an abstraction to share data between any of the contexts
we saw in the previous section (task handlers, `init` and `idle`): resources.

Resources are data visible only to functions declared within the `#[app]`
module. The framework gives the user complete control over which context
can access which resource.

All resources are declared as *two* `struct`s within the `#[app]` module.
Each field in these structures corresponds to a different resource.
One `struct` must be annotated with the attribute `#[local]`.
The other `struct` must be annotated with the attribute `#[shared]`.
The difference between these two sets of resources will be covered later.

Each context (task handler, `init` or `idle`) must declare the resources it
intends to access in its corresponding metadata attribute using either the
`local` or `shared` argument. This argument takes a list of resource names as
its value. The listed resources are made available to the context under the
`local` and `shared` fields of the `Context` structure.

All resources are initialized at runtime, after the `#[init]` function returns.
The `#[init]` function must return the initial values for all resources; hence its return type includes the types of the `#[shared]` and `#[local]` structs.
Because resources are uninitialized during the execution of the `#[init]` function, they cannot be accessed within the `#[init]` function.

The example application shown below contains two interrupt handlers.
Each handler has access to its own `#[local]` resource.

``` rust
{{#include ../../../../examples/resource.rs}}
```

``` console
$ cargo run --example resource
{{#include ../../../../ci/expected/resource.run}}
```

A `#[local]` resource cannot be accessed from outside the task it was associated to in a `#[task]` attribute.
Assigning the same `#[local]` resource to more than one task is a compile-time error.

## `lock`

Critical sections are required to access `#[shared]` resources in a data race-free manner.

The `shared` field of the passed `Context` implements the [`Mutex`] trait for each shared resource accessible to the task.

The only method on this trait, [`lock`], runs its closure argument in a critical section.

[`Mutex`]: ../../../api/rtic/trait.Mutex.html
[`lock`]: ../../../api/rtic/trait.Mutex.html#method.lock

The critical section created by the `lock` API is based on dynamic priorities: it temporarily raises the dynamic priority of the context to a *ceiling* priority that prevents other tasks from preempting the critical section. This synchronization protocol is known as the [Immediate Ceiling Priority Protocol
(ICPP)][icpp], and complies with [Stack Resource Policy(SRP)][srp] based scheduling of RTIC.

[icpp]: https://en.wikipedia.org/wiki/Priority_ceiling_protocol
[srp]: https://en.wikipedia.org/wiki/Stack_Resource_Policy

In the example below we have three interrupt handlers with priorities ranging from one to three. The two handlers with the lower priorities contend for the `shared` resource and need to lock the resource for accessing the data. The highest priority handler, which do not access the `shared` resource, is free to preempt the critical section created by the
lowest priority handler.

``` rust
{{#include ../../../../examples/lock.rs}}
```

``` console
$ cargo run --example lock
{{#include ../../../../ci/expected/lock.run}}
```

## Multi-lock

As an extension to `lock`, and to reduce rightward drift, locks can be taken as tuples. The following examples shows this in use:

``` rust
{{#include ../../../../examples/multilock.rs}}
```

## Only shared (`&-`) access

By default the framework assumes that all tasks require exclusive access (`&mut-`) to resources but it is possible to specify that a task only requires shared access (`&-`) to a resource using the `&resource_name` syntax in the `resources` list.

The advantage of specifying shared access (`&-`) to a resource is that no locks are required to access the resource even if the resource is contended by several tasks running at different priorities. The downside is that the task only gets a shared reference (`&-`) to the resource, limiting the operations it can perform on it, but where a shared reference is enough this approach reduces the number of required locks. In addition to simple immutable data, this shared access can be useful where the resource type safely implements interior mutability, with
appropriate locking or atomic operations of its own.

Note that in this release of RTIC it is not possible to request both exclusive access (`&mut-`) and shared access (`&-`) to the *same* resource from different tasks. Attempting to do so will result in a compile error.

In the example below a key (e.g. a cryptographic key) is loaded (or created) at runtime and then used from two tasks that run at different priorities without any kind of lock.

``` rust
{{#include ../../../../examples/only-shared-access.rs}}
```

``` console
$ cargo run --example only-shared-access
{{#include ../../../../ci/expected/only-shared-access.run}}
```

## Lock-free resource access of mutable resources

A critical section is *not* required to access a `#[shared]` resource that's only accessed by tasks running at the *same* priority.
In this case, you can opt out of the `lock` API by adding the `#[lock_free]` field-level attribute to the resource declaration (see example below).
Note that this is merely a convenience: if you do use the `lock` API, at runtime the framework will *not* produce a critical section.
Also worth noting: using `#[lock_free]` on resources shared by tasks running at different priorities will result in a *compile-time* error -- not using the `lock` API would be a data race in that case.

``` rust
{{#include ../../../../examples/lock-free.rs}}
```

``` console
$ cargo run --example lock-free
{{#include ../../../../ci/expected/lock-free.run}}
```
