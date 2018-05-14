# Periodic tasks

We have seen the `schedule_now` method which is used to schedule tasks to run immediately. RTFM
also allows scheduling tasks to run some time in the future via the `schedule_in` API. In a nutshell
the `schedule_in` lets you schedule a task to run in a certain number of clock (HCLK) cycles in the
future. The offset that the `schedule_in` takes as argument is added to the *scheduled start time*
of the *current* task to compute the scheduled start time of the newly scheduled task. This lets you
create periodic tasks without accumulating drift.

**NOTE** Using the `scheduled_in` API requires enabling the "timer-queue" feature.

Let's look at an example:

``` rust
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm;

use cortex_m_rtfm::app;

app! {
    device: stm32f103xx,

    init: {
        schedule_now: [a],
    },

    tasks: {
        a: {
            schedule_in: [a],
        },
    },
}

fn init(ctxt: init::Context) -> init::LateResources {
    let t = &mut ctxt.threshold;

    ctxt.tasks.a.schedule_now(t, ());
}

// number of clock cycles equivalent to 1 second
const S: u32 = 8_000_000;

fn a(ctxt: a::Context) {
    // `u32` timestamp that corresponds to now
    let now = rtfm::now();

    let t = &mut ctxt.threshold;

    println!("a(ss={}, now={})", ctxt.scheduled_start, now);

    a.tasks.a.schedule_in(t, 1 * S, ());
}
```

This program runs a single task that's executed every second and it prints the following:

``` text
a(ss=0, now=71)
a(ss=8000000, now=8000171)
a(ss=16000000, now=16000171)
```

`init` is not a task but all tasks scheduled from it assume that `init` has a scheduled start of `t
= 0` which represents the time at which `init` ends and all tasks can start. `schedule_now` makes
the scheduled task inherit the scheduled start of the current task; in this case the first instance
of `a` inherits the scheduled start of `init`, that is `t = 0`.

Task `a` schedules itself to run `S` cycles (1 second) in the future. The scheduled start of
its next instance will be its current scheduled start plus `S` cycles. Thus, the second instance of
`a` is scheduled to start at `t = 1 * S`, the third instance is scheduled to start at `t = 2 * S`
and so on. Note that it doesn't matter when or where in the body of `a` `schedule_in` is invoked;
the outcome will be the same.

Now the `scheduled_start` of a task is not the *exact* time at which the task will run -- this can
be seen in the output of the above program: `now` doesn't match the scheduled start. There's some
overhead in the task dispatcher so a task will usually run dozens of cycles after its scheduled
start time. Also, priority based scheduling can make lower priority tasks run much later than their
scheduled start time; for example, imagine the scenario where two tasks have the same scheduled
start but different priorities.

## `scheduled_in` and events

Tasks that spawn from `init` have predictable scheduled starts because `init` itself has a scheduled
start of `t = 0`, but what happens with tasks triggered by events which can start at any time? These
tasks use `rtfm::now()` as an *estimate* of their scheduled start. In the best-case scenario
`rtfm::now()` will be very close to the time at which the event happened. But, if the task has low
priority it may not run until other high priority tasks are done; in this scenario `rtfm::now()`,
and thus the estimated scheduled start, could be very far off from the real time at which the event
happened. Take this in consideration when using `scheduled_in` from tasks triggered by events.
