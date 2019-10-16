# Timer queue

In contrast with the `spawn` API, which immediately spawns a software task onto
the scheduler, the `schedule` API can be used to schedule a task to run some
time in the future.

To use the `schedule` API a monotonic timer must be first defined using the
`monotonic` argument of the `#[app]` attribute. This argument takes a path to a
type that implements the [`Monotonic`] trait. The associated type, `Instant`, of
this trait represents a timestamp in arbitrary units and it's used extensively
in the `schedule` API -- it is suggested to model this type after [the one in
the standard library][std-instant].

Although not shown in the trait definition (due to limitations in the trait /
type system) the subtraction of two `Instant`s should return some `Duration`
type (see [`core::time::Duration`]) and this `Duration` type must implement the
`TryInto<u32>` trait. The implementation of this trait must convert the
`Duration` value, which uses some arbitrary unit of time, into the "system timer
(SYST) clock cycles" time unit. The result of the conversion must be a 32-bit
integer. If the result of the conversion doesn't fit in a 32-bit number then the
operation must return an error, any error type.

[`Monotonic`]: ../../../api/rtfm/trait.Monotonic.html
[std-instant]: https://doc.rust-lang.org/std/time/struct.Instant.html
[`core::time::Duration`]: https://doc.rust-lang.org/core/time/struct.Duration.html

For ARMv7+ targets the `rtfm` crate provides a `Monotonic` implementation based
on the built-in CYCle CouNTer (CYCCNT). Note that this is a 32-bit timer clocked
at the frequency of the CPU and as such it is not suitable for tracking time
spans in the order of seconds.

To be able to schedule a software task from a context the name of the task must
first appear in the `schedule` argument of the context attribute. When
scheduling a task the (user-defined) `Instant` at which the task should be
executed must be passed as the first argument of the `schedule` invocation.

Additionally, the chosen `monotonic` timer must be configured and initialized
during the `#[init]` phase. Note that this is *also* the case if you choose to
use the `CYCCNT` provided by the `cortex-m-rtfm` crate.

The example below schedules two tasks from `init`: `foo` and `bar`. `foo` is
scheduled to run 8 million clock cycles in the future. Next, `bar` is scheduled
to run 4 million clock cycles in the future. Thus `bar` runs before `foo` since
it was scheduled to run first.

> **IMPORTANT**: The examples that use the `schedule` API or the `Instant`
> abstraction will **not** properly work on QEMU because the Cortex-M cycle
> counter functionality has not been implemented in `qemu-system-arm`.

``` rust
{{#include ../../../../examples/schedule.rs}}
```

Running the program on real hardware produces the following output in the
console:

``` text
{{#include ../../../../ci/expected/schedule.run}}
```

When the `schedule` API is being used the runtime internally uses the `SysTick`
interrupt handler and the system timer peripheral (`SYST`) so neither can be
used by the application. This is accomplished by changing the type of
`init::Context.core` from `cortex_m::Peripherals` to `rtfm::Peripherals`. The
latter structure contains all the fields of the former minus the `SYST` one.

## Periodic tasks

Software tasks have access to the `Instant` at which they were scheduled to run
through the `scheduled` variable. This information and the `schedule` API can be
used to implement periodic tasks as shown in the example below.

``` rust
{{#include ../../../../examples/periodic.rs}}
```

This is the output produced by the example. Note that there is zero drift /
jitter even though `schedule.foo` was invoked at the *end* of `foo`. Using
`Instant::now` instead of `scheduled` would have resulted in drift / jitter.

``` text
{{#include ../../../../ci/expected/periodic.run}}
```

## Baseline

For the tasks scheduled from `init` we have exact information about their
`scheduled` time. For hardware tasks there's no `scheduled` time because these
tasks are asynchronous in nature. For hardware tasks the runtime provides a
`start` time, which indicates the time at which the task handler started
executing.

Note that `start` is **not** equal to the arrival time of the event that fired
the task. Depending on the priority of the task and the load of the system the
`start` time could be very far off from the event arrival time.

What do you think will be the value of `scheduled` for software tasks that are
*spawned* instead of scheduled? The answer is that spawned tasks inherit the
*baseline* time of the context that spawned it. The baseline of hardware tasks
is their `start` time, the baseline of software tasks is their `scheduled` time
and the baseline of `init` is the system start time or time zero
(`Instant::zero()`). `idle` doesn't really have a baseline but tasks spawned
from it will use `Instant::now()` as their baseline time.

The example below showcases the different meanings of the *baseline*.

``` rust
{{#include ../../../../examples/baseline.rs}}
```

Running the program on real hardware produces the following output in the console:

``` text
{{#include ../../../../ci/expected/baseline.run}}
```
