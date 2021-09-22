# Monotonic & spawn_{at/after}

The understanding of time is an important concept in embedded systems, and to be able to run tasks
based on time is very useful. For this use-case the framework provides the static methods
`task::spawn_after(/* duration */)` and `task::spawn_at(/* specific time instant */)`.
Mostly one uses `spawn_after`, but in cases where it's needed to have spawns happen without drift or
to a fixed baseline `spawn_at` is available.

To support this the `#[monotonic]` attribute exists which is applied to a type alias definition.
This type alias must point to a type which implements the [`rtic_monotonic::Monotonic`] trait.
This is generally some timer which handles the timing of the system. One or more monotonics can be
used in the same system, for example a slow timer that is used to wake the system from sleep and another
that is used for high granularity scheduling while the system is awake.

[`rtic_monotonic::Monotonic`]: https://docs.rs/rtic-monotonic

The attribute has one required parameter and two optional parameters, `binds`, `default` and
`priority` respectively. `binds = InterruptName` defines which interrupt vector is associated to
the timer's interrupt, `default = true` enables a shorthand API when spawning and accessing the
time (`monotonics::now()` vs `monotonics::MyMono::now()`), and `priority` sets the priority the
interrupt vector has.

> By default `priority` is set to the **maximum priority** of the system but a lower priority
> can be selected if a high priority task cannot take the jitter introduced by the scheduling.
> This can however introduce jitter and delays into the scheduling, making it a trade-off.

Finally, the monotonics must be initialized in `#[init]` and returned in the `init::Monotonic( ... )` tuple.
This moves the monotonics into the active state which makes it possible to use them.

An example is provided below:

``` rust
{{#include ../../../../examples/schedule.rs}}
```

``` console
$ cargo run --target thumbv7m-none-eabi --example message
{{#include ../../../../ci/expected/schedule.run}}
```

## Canceling or rescheduling a scheduled task

Tasks spawned using `task::spawn_after` and `task::spawn_at` has as returns a `SpawnHandle`,
where the `SpawnHandle` can be used to cancel or reschedule a task that will run in the future.
If `cancel` or `reschedule_at`/`reschedule_after` returns an `Err` it means that the operation was
too late and that the task is already sent for execution. The following example shows this in action:

``` rust
{{#include ../../../../examples/cancel-reschedule.rs}}
```

``` console
$ cargo run --target thumbv7m-none-eabi --example message
{{#include ../../../../ci/expected/cancel-reschedule.run}}
```
