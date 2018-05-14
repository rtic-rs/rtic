# Priority based scheduling

We have talked about tasks but we have glossed over how they are scheduled. RTFM uses a priority
based scheduler: tasks with higher priority will preempt lower priority ones. Once a task starts it
runs to completion and will only be suspended if a higher priority task needs to be executed, but
once the higher priority task finishes the lower priority one resumes execution.

Let's illustrate how scheduling works with an example:

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
            // priority: 1,
            schedule_now: [c],
        },
        b: {
            priority: 2,
        },
        c: {
            priority: 3,
            schedule_now: [b],
        },
    },
}

fn init(ctxt: init::Context) -> init::LateResources {
    let t = &mut ctxt.threshold;

    println!("IN1");

    ctxt.tasks.a.schedule_now(t, ());

    println!("IN2");

    init::LateResources {}
}

fn idle(ctxt: idle::Context) -> ! {
    println!("ID");

    loop {
        // ..
    }
}

fn a(ctxt: a::Context) {
    let t = &mut ctxt.threshold;

    println!("A1");

    ctxt.tasks.c.schedule_now(t, ());

    println!("A2");
}

fn b(ctxt: b::Context) {
    let t = &mut ctxt.threshold;

    println!("B");
}

fn c(ctxt: c::Context) {
    let t = &mut ctxt.threshold;

    println!("C1");

    ctxt.tasks.b.schedule_now(t, ());

    println!("C2");
}
```

This program prints:

``` text
IN1
IN2
A1
C1
C2
B
A2
ID
```

The RTFM scheduler is actually hardware based and built on top of the NVIC (Nested Vector Interrupt
Controller) peripheral and the interrupt mechanism of the Cortex-M architecture so tasks can't
run while the interrupts are disabled. Thus tasks scheduled during `init` won't run until *after*
`init` ends regardless of their priority.

The program execution goes like this:

- `init` prints "I1". Then task `a` is scheduled to run immediately but nothing happens because
  interrupts are disabled. `init` prints "I2".

- `init` ends and now tasks can run. Task `a` preempts `idle`, which runs after `init` . `idle`
  is not a task per se because it's never ending, but it has the lowest priority (priority = 0) so
  all tasks can preempt it -- all tasks have a priority of 1 or larger.

- Task `a` prints "A1" and then schedules task `c` to run immediately. Because task `c` has higher
  priority than task `a` it preempts `a`.

- Task `c` starts and print "C1". Then it schedules task `b` to run immediately. Because task `b`
  has lower priority than `c` it gets postponed. Task `c` prints "C2" and returns.

- After task `c` ends task `a` should be resumed but task `b` is pending and has higher priority so
  task `b` preempts `a`. Task `b` prints "B" and ends.

- Task `a` is finally resumed. Task `a` prints "A2" and returns.

- After task `a` ends there's no task pending execution so `idle` is resumed. `idle` prints "ID" and
  then executes some infinite `loop`.
