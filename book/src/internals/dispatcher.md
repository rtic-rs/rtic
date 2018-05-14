# Dispatching tasks

Let's first analyze the simpler case of dispatching tasks with `input` type of `()`, i.e. the
message contained no payload, and was scheduled using `schedule_now`.

All tasks scheduled by other tasks, i.e. tasks not bound to an interrupt, that are to be executed at
the same priority are dispatched from the same *task dispatcher*. Task dispatchers are implemented
on top of the free interrupt handlers which are declared in `free_interrupts`. Each task dispatcher
has a queue of tasks ready to execute -- this queues are called *ready queues*. RTFM uses
`heapless::RingBuffer` for all the internal queues; these queues are lock-free and wait-free when
the queue has a single consumer and a single producer.

Let's illustrate the workings of task dispatchers with an example. Assume we have an application
with 4 tasks not bound to interrupts: two of them, `a` and `b`, are dispatched at priority 1; and
the other two, `c` and `d`, are dispatched at priority 2. This is what the task dispatchers produced
by the `app!` macro look like:

``` rust
// priority = 1
unsafe extern "C" fn EXTI0() {
    while let Some(task) = __1::READY_QUEUE.dequeue() {
        match task {
            __1::Task::a => a(a::Context::new()),
            __1::Task::b => b(b::Context::new()),
        }
    }
}

// priority = 2
unsafe extern "C" fn EXTI1() {
    while let Some(task) = __2::READY_QUEUE.dequeue() {
        match task {
            __2::Task::c => c(c::Context::new()),
            __2::Task::d => d(d::Context::new()),
        }
    }
}

mod __1 {
    // Tasks dispatched at priority = 1
    enum Task { a, b }

    static mut READY_QUEUE: Queue<Task, UN> = Queue::new();
}

mod __2 {
    // Tasks dispatched at priority = 2
    enum Task { c, d }

    static mut READY_QUEUE: Queue<Task, UN> = Queue::new();
}
```

Note that we have two queues here: one for priority = 1 and another for priority = 2. The
interrupts used to dispatch tasks are chosen from the list of `free_interrupts` declared in the
`app!` macro.

#### Payloads

Now let's add payloads to the messages. The message queues will now not only store the task name
(`enum Task`) but also an *index* (`u8`) to the payload.

Let's first look at how the first task dispatcher changed: let's say that tasks `a` and `b` now
expect payloads of `i32` and `i16`, respectively.

``` rust
mod a {
    static mut PAYLOADS: [i32; N] = unsafe { uninitialized() };

    static mut FREE_QUEUE: Queue<u8, UN> = Queue::new();

    // ..
}

mod b {
    static mut PAYLOADS: [i16; N] = unsafe { uninitialized() };

    static mut FREE_QUEUE: Queue<u8, UN> = Queue::new();

    // ..
}

mod __1 {
    // Tasks dispatched at priority = 1
    enum Task { a, b }

    static mut READY_QUEUE: Queue<(Task, u8), UN> = Queue::new();
}

mod __2 {
    // Tasks dispatched at priority = 2
    enum Task { c, d }

    static mut READY_QUEUE: Queue<(Task, u8), UN> = Queue::new();
}

// priority = 1
unsafe extern "C" fn EXTI0() {
    while let Some(task) = READY_QUEUE.dequeue() {
        match (task, index) {
            __1::Task::a => {
                let payload: i32 = ptr::read(&a::PAYLOADS[index]);
                a::FREE_QUEUE.enqueue_unchecked(index);

                a(a::Context::new(payload))
            },
            __2::Task::b => {
                let payload: i16 = ptr::read(&b::PAYLOADS[index]);
                b::FREE_QUEUE.enqueue_unchecked(index);

                b(b::Context::new(payload))
            },
        }
    }
}
```

Each task dispatcher continuously dequeues tasks from the ready queue until it's empty. After
dequeuing a task - index pair the task dispatcher looks at which task it has to execute (`match`)
and uses this information to fetch (`ptr::read`) the payload from the corresponding list of
payloads (`PAYLOADS`) -- there's one such list per task. After retrieving the payload this leaves an
empty slot in the list of payloads; the index to this empty slot is appended to a list of free slots
(`FREE_QUEUE`). Finally, the task dispatcher proceed to execute the task using the message payload
as the input.
