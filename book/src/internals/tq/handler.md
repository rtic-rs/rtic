# The timer queue handler

The `SysTick` exception handler is used as the timer queue handler. This handler takes cares of
moving tasks that have become ready from the timer queue to their respective ready queues. The
timer queue makes use of the Cortex-M sytem timer, the `SysTick`, to schedule when the `SysTick`
handler should run.

This is what the `SYS_TICK` handler looks like for our running example where `a` and `d` are
scheduled via `scheduled_after`:

``` rust
unsafe extern "C" fn SYS_TICK() {
    let mut t = Threshold::new(..);

    loop {
        let next = TQ.claim_mut(&mut t, |tq, _| {
            let front = tq.priority_queue.peek().map(|nr| nr.scheduled_start);

            if let Some(scheduled_start) = front {
                let diff = scheduled_start - Instant::now();

                if diff > 0 {
                    // task still not ready, schedule this handler to run in the future by
                    // setting a new timeout

                    // maximum timeout supported by the SysTick
                    const MAX: u32 = 0x00ffffff;

                    SYST.set_reload(cmp::min(MAX, diff as u32));

                    // start counting from the new reload
                    SYST.clear_current();

                    None
                } else {
                    // task became ready
                    let nr = tq.priority_queue.pop_unchecked();

                    Some((nr.task, nr.index))
                }
            } else {
                // the queue is empty
                SYST.disable_interrupt();

                None
            }
        });

        if let Some((task, index)) = next {
            // place the tasks - index pair into the corresponding ready queue
            match task {
                __tq::Task::a => {
                    __1::READY_QUEUE.claim_mut(t, |rq, _| {
                        rq.enqueue_unchecked((__1::Task::a, index));
                    });

                    NVIC.set_pending(Interrupt::EXTI0);
                },
                __tq::Task::d => {
                    __2::READY_QUEUE.claim_mut(t, |rq, _| {
                        rq.enqueue_unchecked((__2::Task::d, index));
                    });

                    NVIC.set_pending(Interrupt::EXTI1);
                },
            }
        } else {
            return;
        }
    }
}
```

The `SYS_TICK` handler will use a `loop` to move all the tasks that have become ready from the
priority queue, the timer queue, to the ready queues.

To do that the handler will check the front of the priority queue, which contains the task with the
closest `scheduled_start`. If the queue is empty then the handler will disable the `SysTick`
exception and return; the handler won't run again until the exception is re-enabled by
`TimerQueue.enqueue`.

If the priority queue was not empty then the handler will then compare that closest
`scheduled_start` against the current time (`Instant::now()`). If the `scheduled_start` time has not
been reached the handler will schedule to run itself in the future by setting a `SysTick` timeout.
If instead we are past the closest `scheduled_start` then the handler will move the task at the
front of the queue to its corresponding `READY_QUEUE` and set the corresponding task dispatcher as
pending.
