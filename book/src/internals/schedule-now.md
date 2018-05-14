# `schedule_now`

We saw how tasks dispatching works; now let's see how `schedule_now` is implemented. Assume that
task `a` can be `schedule_now`-ed by task `b`; in this scenario the `app!` macro generates code like
this:

``` rust
mod __schedule_now {
    pub struct a { _not_send_or_sync: PhantomData<*const ()> }

    impl a {
        fn schedule_now(&mut self, t: &mut Threshold, payload: i32) -> Result<(), i32> {
            if let Some(index) = a::FREE_QUEUE.claim_mut(t, |fq, _| fq.dequeue()) {
                ptr::write(&mut a::PAYLOADS[index], payload);

                __1::READY_QUEUE.claim_mut(t, |rq, _| {
                    rq.enqueue_unchecked((__1::Task::A, index))
                });

                NVIC.set_pending(Interrupt::EXTI0);
            } else {
                Err(payload)
            }
        }
    }
}

mod b {
    pub struct Tasks { a: __schedule_now::a }

    pub struct Context {
        tasks: Tasks,
        // ..
    }
}
```

The first thing to do to schedule a new task is to get a free slot, where to store the payload, from
the `FREE_QUEUE`. If the list of payloads (`PAYLOADS`) is full, i.e. if `FREE_QUEUE` is empty, then
`schedule_now` early returns with an error. After retrieving a free slot the `payload` is stored
into it. Then the task - index pair is enqueued into the corresponding priority queue. Finally, the
interrupt whose handler is being used as task dispatcher is set as *pending* -- this will cause the
`NVIC` (the hardware scheduler) to execute the handler.

Fetching a free slot from the free queue and enqueuing a task - index pair into the ready queue may
require critical sections so the queues are accessed as resources using `claim_mut`. In a later
section we'll analyze where critical sections are required.
