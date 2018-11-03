## Resources

One of the limitations of the attributes provided by the `cortex-m-rt` crate is
that sharing data (or peripherals) between interrupts, or between an interrupt
and the `entry` function, requires a `cortex_m::interrupt::Mutex`, which
*always* requires disabling *all* interrupts to access the data. Disabling all
the interrupts is not always required for memory safety but the compiler doesn't
have enough information to optimize the access to the shared data.

The `app` attribute has a full view of the application thus it can optimize
access to `static` variables. In RTFM we refer to the `static` variables
declared inside the `app` pseudo-module as *resources*. To access a resource the
context (`init`, `idle`, `interrupt` or `exception`) must first declare the
resource in the `resources` argument of its attribute.

In the example below two interrupt handlers access the same resource. No `Mutex`
is required in this case because the two handlers run at the same priority and
no preemption is possible. The `SHARED` resource can only be accessed by these
two handlers.

``` rust
{{#include ../../../examples/resource.rs}}
```

``` console
$ cargo run --example resource
{{#include ../../../ci/expected/resource.run}}```

## Priorities

The priority of each handler can be declared in the `interrupt` and `exception`
attributes. It's not possible to set the priority in any other way because the
runtime takes ownership of the `NVIC` peripheral; it's also not possible to
change the priority of a handler / task at runtime. Thanks to this restriction
the framework has knowledge about the *static* priorities of all interrupt and
exception handlers.

Interrupts and exceptions can have priorities in the range `1..=(1 <<
NVIC_PRIO_BITS)` where `NVIC_PRIO_BITS` is a constant defined in the `device`
crate. The `idle` task has a priority of `0`, the lowest priority.

Resources that are shared between handlers that run at different priorities
require critical sections for memory safety. The framework ensures that critical
sections are used but *only where required*: for example, no critical section is
required by the highest priority handler that has access to the resource.

The critical section API provided by the RTFM framework (see [`Mutex`]) is
based on dynamic priorities rather than on disabling interrupts. The consequence
is that these critical sections will prevent *some* handlers, including all the
ones that contend for the resource, from *starting* but will let higher priority
handlers, that don't contend for the resource, run.

[`Mutex`]: ../../api/rtfm/trait.Mutex.html

In the example below we have three interrupt handlers with priorities ranging
from one to three. The two handlers with the lower priorities contend for the
`SHARED` resource. The lowest priority handler needs to [`lock`] the
`SHARED` resource to access its data, whereas the mid priority handler can
directly access its data. The highest priority handler is free to preempt
the critical section created by the lowest priority handler.

[`lock`]: ../../api/rtfm/trait.Mutex.html#method.lock

``` rust
{{#include ../../../examples/lock.rs}}
```

``` console
$ cargo run --example lock
{{#include ../../../ci/expected/lock.run}}```

## Late resources

Unlike normal `static` variables, which need to be assigned an initial value
when declared, resources can be initialized at runtime. We refer to these
runtime initialized resources as *late resources*. Late resources are useful for
*moving* (as in transferring ownership) peripherals initialized in `init` into
interrupt and exception handlers.

Late resources are declared like normal resources but that are given an initial
value of `()` (the unit value). Late resources must be initialized at the end of
the `init` function using plain assignments (e.g. `FOO = 1`).

The example below uses late resources to stablish a lockless, one-way channel
between the `UART0` interrupt handler and the `idle` function. A single producer
single consumer [`Queue`] is used as the channel. The queue is split into
consumer and producer end points in `init` and then each end point is stored
in a different resource; `UART0` owns the producer resource and `idle` owns
the consumer resource.

[`Queue`]: ../../api/heapless/spsc/struct.Queue.html

``` rust
{{#include ../../../examples/late.rs}}
```

``` console
$ cargo run --example late
{{#include ../../../ci/expected/late.run}}```

## `static` resources

`static` variables can also be used as resources. Tasks can only get `&`
(shared) references to these resources but locks are never required to access
their data. You can think of `static` resources as plain `static` variables that
can be initialized at runtime and have better scoping rules: you can control
which tasks can access the variable, instead of the variable being visible to
all the functions in the scope it was declared in.

In the example below a key is loaded (or created) at runtime and then used from
two tasks that run at different priorities.

``` rust
{{#include ../../../examples/static.rs}}
```

``` console
$ cargo run --example static
{{#include ../../../ci/expected/static.run}}```
