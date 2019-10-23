# Timer queue

When the `timer-queue` feature is enabled the RTFM framework includes a *global
timer queue* that applications can use to *schedule* software tasks to run at
some time in the future.

> **NOTE**: The timer-queue feature can't be enabled when the target is
> `thumbv6m-none-eabi` because there's no timer queue support for ARMv6-M. This
> may change in the future.

> **NOTE**: When the `timer-queue` feature is enabled you will *not* be able to
> use the `SysTick` exception as a hardware task because the runtime uses it to
> implement the global timer queue.

To be able to schedule a software task the name of the task must appear in the
`schedule` argument of the context attribute. When scheduling a task the
[`Instant`] at which the task should be executed must be passed as the first
argument of the `schedule` invocation.

[`Instant`]: ../../api/rtfm/struct.Instant.html

The RTFM runtime includes a monotonic, non-decreasing, 32-bit timer which can be
queried using the `Instant::now` constructor. A [`Duration`] can be added to
`Instant::now()` to obtain an `Instant` into the future. The monotonic timer is
disabled while `init` runs so `Instant::now()` always returns the value
`Instant(0 /* clock cycles */)`; the timer is enabled right before the
interrupts are re-enabled and `idle` is executed.

[`Duration`]: ../../api/rtfm/struct.Duration.html

The example below schedules two tasks from `init`: `foo` and `bar`. `foo` is
scheduled to run 8 million clock cycles in the future. Next, `bar` is scheduled
to run 4 million clock cycles in the future. `bar` runs before `foo` since it
was scheduled to run first.

> **IMPORTANT**: The examples that use the `schedule` API or the `Instant`
> abstraction will **not** properly work on QEMU because the Cortex-M cycle
> counter functionality has not been implemented in `qemu-system-arm`.

``` rust
{{#include ../../../../examples/schedule.rs}}
```

Running the program on real hardware produces the following output in the console:

``` text
{{#include ../../../../ci/expected/schedule.run}}
```

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
is `start`, the baseline of software tasks is `scheduled` and the baseline of
`init` is `start = Instant(0)`. `idle` doesn't really have a baseline but tasks
spawned from it will use `Instant::now()` as their baseline time.

The example below showcases the different meanings of the *baseline*.

``` rust
{{#include ../../../../examples/baseline.rs}}
```

Running the program on real hardware produces the following output in the console:

``` text
{{#include ../../../../ci/expected/baseline.run}}
```

## Caveats

The `Instant` and `Duration` APIs are meant to be exclusively used with the
`schedule` API to schedule tasks *with a precision of a single core clock
cycle*. These APIs are *not*, for example, meant to be used to time external
events like a user pressing a button.

The timer queue feature internally uses the system timer, `SysTick`. This timer
is a 24-bit counter and it's clocked at the core clock frequency so tasks
scheduled more than `(1 << 24).cycles()` in the future will incur in additional
overhead, proportional to the size of their `Duration`, compared to task
scheduled with `Duration`s below that threshold.

If you need periodic tasks with periods greater than `(1 << 24).cycles()` you
likely don't need a timer with a resolution of one core clock cycle so we advise
you instead use a device timer with an appropriate prescaler.

We can't stop you from using `Instant` to measure external events so please be
aware that `Instant.sub` / `Instant.elapsed` will never return a `Duration`
equal or greater than `(1 << 31).cycles()` so you won't be able to measure
events that last more than `1 << 31` core clock cycles (e.g. seconds).

Adding a `Duration` equal or greater than `(1 << 31).cycles()` to an `Instant`
will effectively overflow it so it's not possible to schedule a task more than
`(1 << 31).cycles()` in the future. There are some debug assertions in place to
catch this kind of user error but it's not possible to prevent it with 100%
success rate because one can always write `(instant + duration) + duration` and
bypass the runtime checks.
