# Timer queue

The timer queue functionality lets the user schedule tasks to run at some time
in the future. Unsurprisingly, this feature is also implemented using a queue:
a priority queue where the scheduled tasks are kept sorted by earliest scheduled
time. This feature requires a timer capable of setting up timeout interrupts.
The timer is used to trigger an interrupt when the scheduled time of a task is
up; at that point the task is removed from the timer queue and moved into the
appropriate ready queue.

Let's see how this in implemented in code. Consider the following program:

``` rust
#[rtfm::app(device = ..)]
const APP: () = {
    // ..

    #[task(capacity = 2, schedule = [foo])]
    fn foo(c: foo::Context, x: u32) {
        // schedule this task to run again in 1M cycles
        c.schedule.foo(c.scheduled + Duration::cycles(1_000_000), x + 1).ok();
    }

    extern "C" {
        fn UART0();
    }
};
```

## `schedule`

Let's first look at the `schedule` API.

``` rust
mod foo {
    pub struct Schedule<'a> {
        priority: &'a Cell<u8>,
    }

    impl<'a> Schedule<'a> {
        // unsafe and hidden because we don't want the user to tamper with this
        #[doc(hidden)]
        pub unsafe fn priority(&self) -> &Cell<u8> {
            self.priority
        }
    }
}

const APP: () = {
    type Instant = <path::to::user::monotonic::timer as rtfm::Monotonic>::Instant;

    // all tasks that can be `schedule`-d
    enum T {
        foo,
    }

    struct NotReady {
        index: u8,
        instant: Instant,
        task: T,
    }

    // The timer queue is a binary (min) heap of `NotReady` tasks
    static mut TQ: TimerQueue<U2> = ..;
    const TQ_CEILING: u8 = 1;

    static mut foo_FQ: Queue<u8, U2> = Queue::new();
    const foo_FQ_CEILING: u8 = 1;

    static mut foo_INPUTS: [MaybeUninit<u32>; 2] =
        [MaybeUninit::uninit(), MaybeUninit::uninit()];

    static mut foo_INSTANTS: [MaybeUninit<Instant>; 2] =
        [MaybeUninit::uninit(), MaybeUninit::uninit()];

    impl<'a> foo::Schedule<'a> {
        fn foo(&self, instant: Instant, input: u32) -> Result<(), u32> {
            unsafe {
                let priority = self.priority();
                if let Some(index) = lock(priority, foo_FQ_CEILING, || {
                    foo_FQ.split().1.dequeue()
                }) {
                    // `index` is an owning pointer into these buffers
                    foo_INSTANTS[index as usize].write(instant);
                    foo_INPUTS[index as usize].write(input);

                    let nr = NotReady {
                        index,
                        instant,
                        task: T::foo,
                    };

                    lock(priority, TQ_CEILING, || {
                        TQ.enqueue_unchecked(nr);
                    });
                } else {
                    // No space left to store the input / instant
                    Err(input)
                }
            }
        }
    }
};
```

This looks very similar to the `Spawn` implementation. In fact, the same
`INPUTS` buffer and free queue (`FQ`) are shared between the `spawn` and
`schedule` APIs. The main difference between the two is that `schedule` also
stores the `Instant` at which the task was scheduled to run in a separate buffer
(`foo_INSTANTS` in this case).

`TimerQueue::enqueue_unchecked` does a bit more work that just adding the entry
into a min-heap: it also pends the system timer interrupt (`SysTick`) if the new
entry ended up first in the queue.

## The system timer

The system timer interrupt (`SysTick`) takes cares of two things: moving tasks
that have become ready from the timer queue into the right ready queue and
setting up a timeout interrupt to fire when the scheduled time of the next task
is up.

Let's see the associated code.

``` rust
const APP: () = {
    #[no_mangle]
    fn SysTick() {
        const PRIORITY: u8 = 1;

        let priority = &Cell::new(PRIORITY);
        while let Some(ready) = lock(priority, TQ_CEILING, || TQ.dequeue()) {
            match ready.task {
                T::foo => {
                    // move this task into the `RQ1` ready queue
                    lock(priority, RQ1_CEILING, || {
                        RQ1.split().0.enqueue_unchecked(Ready {
                           task: T1::foo,
                           index: ready.index,
                        })
                    });

                    // pend the task dispatcher
                    rtfm::pend(Interrupt::UART0);
                }
            }
        }
    }
};
```

This looks similar to a task dispatcher except that instead of running the
ready task this only places the task in the corresponding ready queue, that
way it will run at the right priority.

`TimerQueue::dequeue` will set up a new timeout interrupt when it returns
`None`. This ties in with `TimerQueue::enqueue_unchecked`, which pends this
handler; basically, `enqueue_unchecked` delegates the task of setting up a new
timeout interrupt to the `SysTick` handler.

## Resolution and range of `cyccnt::Instant` and `cyccnt::Duration`

RTFM provides a `Monotonic` implementation based on the `DWT`'s (Data Watchpoint
and Trace) cycle counter. `Instant::now` returns a snapshot of this timer; these
DWT snapshots (`Instant`s) are used to sort entries in the timer queue. The
cycle counter is a 32-bit counter clocked at the core clock frequency. This
counter wraps around every `(1 << 32)` clock cycles; there's no interrupt
associated to this counter so nothing worth noting happens when it wraps around.

To order `Instant`s in the queue we need to compare two 32-bit integers. To
account for the wrap-around behavior we use the difference between two
`Instant`s, `a - b`, and treat the result as a 32-bit signed integer. If the
result is less than zero then `b` is a later `Instant`; if the result is greater
than zero then `b` is an earlier `Instant`. This means that scheduling a task at
an `Instant` that's `(1 << 31) - 1` cycles greater than the scheduled time
(`Instant`) of the first (earliest) entry in the queue will cause the task to be
inserted at the wrong place in the queue. There some debug assertions in place
to prevent this user error but it can't be avoided because the user can write
`(instant + duration_a) + duration_b` and overflow the `Instant`.

The system timer, `SysTick`, is a 24-bit counter also clocked at the core clock
frequency. When the next scheduled task is more than `1 << 24` clock cycles in
the future an interrupt is set to go off in `1 << 24` cycles. This process may
need to happen several times until the next scheduled task is within the range
of the `SysTick` counter.

In conclusion, both `Instant` and `Duration` have a resolution of 1 core clock
cycle and `Duration` effectively has a (half-open) range of `0..(1 << 31)` (end
not included) core clock cycles.

## Queue capacity

The capacity of the timer queue is chosen to be the sum of the capacities of all
`schedule`-able tasks. Like in the case of the ready queues, this means that
once we have claimed a free slot in the `INPUTS` buffer we are guaranteed to be
able to insert the task in the timer queue; this lets us omit runtime checks.

## System timer priority

The priority of the system timer can't set by the user; it is chosen by the
framework. To ensure that lower priority tasks don't prevent higher priority
tasks from running we choose the priority of the system timer to be the maximum
of all the `schedule`-able tasks.

To see why this is required consider the case where two previously scheduled
tasks with priorities `2` and `3` become ready at about the same time but the
lower priority task is moved into the ready queue first. If the system timer
priority was, for example, `1` then after moving the lower priority (`2`) task
it would run to completion (due to it being higher priority than the system
timer) delaying the execution of the higher priority (`3`) task. To prevent
scenarios like these the system timer must match the highest priority of the
`schedule`-able tasks; in this example that would be `3`.

## Ceiling analysis

The timer queue is a resource shared between all the tasks that can `schedule` a
task and the `SysTick` handler. Also the `schedule` API contends with the
`spawn` API over the free queues. All this must be considered in the ceiling
analysis.

To illustrate, consider the following example:

``` rust
#[rtfm::app(device = ..)]
const APP: () = {
    #[task(priority = 3, spawn = [baz])]
    fn foo(c: foo::Context) {
        // ..
    }

    #[task(priority = 2, schedule = [foo, baz])]
    fn bar(c: bar::Context) {
        // ..
    }

    #[task(priority = 1)]
    fn baz(c: baz::Context) {
        // ..
    }
};
```

The ceiling analysis would go like this:

- `foo` (prio = 3) and `baz` (prio = 1) are `schedule`-able task so the
  `SysTick` must run at the highest priority between these two, that is `3`.

- `foo::Spawn` (prio = 3) and `bar::Schedule` (prio = 2) contend over the
  consumer endpoind of `baz_FQ`; this leads to a priority ceiling of `3`.

- `bar::Schedule` (prio = 2) has exclusive access over the consumer endpoint of
`foo_FQ`; thus the priority ceiling of `foo_FQ` is effectively `2`.

- `SysTick` (prio = 3) and `bar::Schedule` (prio = 2) contend over the timer
  queue `TQ`; this leads to a priority ceiling of `3`.

- `SysTick` (prio = 3) and `foo::Spawn` (prio = 3) both have lock-free access to
  the ready queue `RQ3`, which holds `foo` entries; thus the priority ceiling of
  `RQ3` is effectively `3`.

- The `SysTick` has exclusive access to the ready queue `RQ1`, which holds `baz`
  entries; thus the priority ceiling of `RQ1` is effectively `3`.

## Changes in the `spawn` implementation

When the `schedule` API is used the `spawn` implementation changes a bit to
track the baseline of tasks. As you saw in the `schedule` implementation there's
an `INSTANTS` buffers used to store the time at which a task was scheduled to
run; this `Instant` is read in the task dispatcher and passed to the user code
as part of the task context.

``` rust
const APP: () = {
    // ..

    #[no_mangle]
    unsafe UART1() {
        const PRIORITY: u8 = 1;

        let snapshot = basepri::read();

        while let Some(ready) = RQ1.split().1.dequeue() {
            match ready.task {
                Task::baz => {
                    let input = baz_INPUTS[ready.index as usize].read();
                    // ADDED
                    let instant = baz_INSTANTS[ready.index as usize].read();

                    baz_FQ.split().0.enqueue_unchecked(ready.index);

                    let priority = Cell::new(PRIORITY);
                    // CHANGED the instant is passed as part the task context
                    baz(baz::Context::new(&priority, instant), input)
                }

                Task::bar => {
                    // looks just like the `baz` branch
                }

            }
        }

        // BASEPRI invariant
        basepri::write(snapshot);
    }
};
```

Conversely, the `spawn` implementation needs to write a value to the `INSTANTS`
buffer. The value to be written is stored in the `Spawn` struct and its either
the `start` time of the hardware task or the `scheduled` time of the software
task.

``` rust
mod foo {
    // ..

    pub struct Spawn<'a> {
        priority: &'a Cell<u8>,
        // ADDED
        instant: Instant,
    }

    impl<'a> Spawn<'a> {
        pub unsafe fn priority(&self) -> &Cell<u8> {
            &self.priority
        }

        // ADDED
        pub unsafe fn instant(&self) -> Instant {
            self.instant
        }
    }
}

const APP: () = {
    impl<'a> foo::Spawn<'a> {
        /// Spawns the `baz` task
        pub fn baz(&self, message: u64) -> Result<(), u64> {
            unsafe {
                match lock(self.priority(), baz_FQ_CEILING, || {
                    baz_FQ.split().1.dequeue()
                }) {
                    Some(index) => {
                        baz_INPUTS[index as usize].write(message);
                        // ADDED
                        baz_INSTANTS[index as usize].write(self.instant());

                        lock(self.priority(), RQ1_CEILING, || {
                            RQ1.split().1.enqueue_unchecked(Ready {
                                task: Task::foo,
                                index,
                            });
                        });

                        rtfm::pend(Interrupt::UART0);
                    }

                    None => {
                        // maximum capacity reached; spawn failed
                        Err(message)
                    }
                }
            }
        }
    }
};
```
