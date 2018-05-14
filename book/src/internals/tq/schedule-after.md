# `schedule_after`

Let's see how `schedule_after` adds tasks to the timer queue.


``` rust
mod __schedule_after {
    impl a {
        fn schedule_after(
            &mut self,
            t: &mut Threshold,
            offset: u32,
            payload: i32,
        ) -> Result<(), i32> {
            if let Some(index) = a::FREE_QUEUE.dequeue() {
                core::ptr::write(
                    &mut a::PAYLOADS[index as usize],
                    payload,
                );

                let scheduled_start = self.scheduled_start + offset;

                core::ptr::write(
                    &mut a::SCHEDULED_STARTS[index as usize],
                    scheduled_start,
                );

                let not_ready = NotReady {
                    index,
                    scheduled_start,
                    task: __tq::Task::a,
                };

                __tq::TIMER_QUEUE.claim_mut(t, |tq, _| tq.enqueue(not_ready));
            } else {
                Err(payload)
            }
        }
    }
}
```

Like `schedule_now`, `schedule_after` starts by fetching a free slot from the `FREE_QUEUE`. If
there's no free slot available the function early returns with an error. Once a free slot (`index`)
has been retrieved the payload is stored in that spot of the payload list (`PAYLOADS`). The
`scheduled_start` of the newly scheduled task is the `scheduled_start` time of the current task plus
the specified `offset`. This `scheduled_start` value is also stored in a list (`SCHEDULED_STARTS`)
at the free slot `index`.  After that's done, the not ready task -- represented by the `NotReady`
struct which contains the `Task` name, the payload / `scheduled_after` index and the actual
`scheduled_start` value -- is inserted in the timer queue.

`TimerQueue.enqueue` does a bit more of work than just adding the not ready task to the priority
queue of tasks:

``` rust
struct TimerQueue {
    priority_queue: BinaryHeap<..>,
}

impl TimerQueue {
    unsafe fn enqueue(&mut self, new: NotReady) {
        let mut is_empty = true;

        if self.priority_queue
            .peek()
            .map(|head| {
                is_empty = false;
                new.scheduled_start < head.scheduled_start
            })
            .unwrap_or(true)
        {
            if is_empty {
                SYST.enable_interrupt();
            }

            SCB.set_pending(Exception::SysTick);
        }

        self.priority_queue.push_unchecked(new);
    }
}
```

If the priority queue is empty or the new not ready task is scheduled to run *before* the current
task at the front of the queue then the `SysTick` exception handler is also enabled and set as
pending. In the next section we'll see the role that this handler plays.

Another important thing to note is that the `Task` enum used in the `NotReady` struct: it only
contains tasks which can be scheduled via `scheduled_after`. The tasks in this set not necessarily
are to be dispatched at the same priority.

Consider the following task configuration:

- Tasks `a` and `b` are dispatched at priority 1
- Tasks `c` and `d` are dispatched at priority 2
- `a` is scheduled using `schedule_after`
- `b` is scheduled using `schedule_now`
- `c` is scheduled using `schedule_now`
- `d` is scheduled both via `schedule_now` and `scheduled_after`

RTFM will end up creating the following `enum`s:

``` rust
mod __1 {
    enum Task { a, b }
}

mod __2 {
    enum Task { c, d }
}

mod __tq {
    enum Task { a, d }
}
```
