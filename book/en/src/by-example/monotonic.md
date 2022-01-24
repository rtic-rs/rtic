# Monotonic & spawn_{at/after}

The understanding of time is an important concept in embedded systems, and to be able to run tasks
based on time is useful. For this use-case the framework provides the static methods
`task::spawn_after(/* duration */)` and `task::spawn_at(/* specific time instant */)`.
`spawn_after` is more commonly used, but in cases where it's needed to have spawns happen
without drift or to a fixed baseline `spawn_at` is available.

The `#[monotonic]` attribute, applied to a type alias definition, exists to support this.
This type alias must point to a type which implements the [`rtic_monotonic::Monotonic`] trait.
This is generally some timer which handles the timing of the system.
One or more monotonics can coexist in the same system, for example a slow timer that wakes the
system from sleep and another which purpose is for fine grained scheduling while the
system is awake.

[`rtic_monotonic::Monotonic`]: https://docs.rs/rtic-monotonic

The attribute has one required parameter and two optional parameters, `binds`, `default` and
`priority` respectively.
The required parameter, `binds = InterruptName`, associates an interrupt vector to the timer's
interrupt, while `default = true` enables a shorthand API when spawning and accessing
time (`monotonics::now()` vs `monotonics::MyMono::now()`), and `priority` sets the priority
of the interrupt vector.

> The default `priority` is the **maximum priority** of the system.
> If your system has a high priority task with tight scheduling requirements,
> it might be desirable to demote the `monotonic` task to a lower priority
> to reduce scheduling jitter for the high priority task.
> This however might introduce jitter and delays into scheduling via the `monotonic`,
> making it a trade-off.

The monotonics are initialized in `#[init]` and returned within the `init::Monotonic( ... )` tuple.
This activates the monotonics making it possible to use them.

See the following example:

``` rust
{{#include ../../../../examples/schedule.rs}}
```

``` console
$ cargo run --target thumbv7m-none-eabi --example schedule
{{#include ../../../../ci/expected/schedule.run}}
```

## Canceling or rescheduling a scheduled task

Tasks spawned using `task::spawn_after` and `task::spawn_at` returns a `SpawnHandle`,
which allows canceling or rescheduling of the task scheduled to run in the future.
If `cancel` or `reschedule_at`/`reschedule_after` returns an `Err` it means that the operation was
too late and that the task is already sent for execution. The following example shows this in action:

``` rust
{{#include ../../../../examples/cancel-reschedule.rs}}
```

``` console
$ cargo run --target thumbv7m-none-eabi --example cancel-reschedule
{{#include ../../../../ci/expected/cancel-reschedule.run}}
```
