# Message passing

So far we have seen tasks as a way to respond to events but events are not the only way to start a
task. A task can schedule another task, optionally passing a message to it.

For example, consider the following application where data is received from the serial interface and
collected into a buffer. `\n` is used as a frame delimiter; once a frame has been received we want
to process the buffer contents but we don't want to do that in the `usart1` task because that task
has to keep up with the fast incoming data and it should be short and high priority. So, instead we
*send* the frame to a *lower priority* task for further processing; this way we keep the `usart1`
task responsive.

``` rust
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm;
extern crate heapless;

use cortex_m_rtfm::app;
use heapless::Vec;
use heapless::consts::*;

app! {
    device: stm32f103xx,

    resources: {
        // 128-byte buffer
        static BUFFER: Vec<u8, U128> = Vec::new();

        // omitted: other resources
    },

    tasks: {
        // task bound to an interrupt
        usart1: {
            // event = data arrived via the serial interface
            interrupt: USART1,

            // higher priority number = more urgent
            priority: 2,

            // omitted: the exact list of resources assigned to this task

            // tasks that this task can schedule
            schedule_now: [process],
        },

        // task schedulable by other tasks
        process: {
            // the input this task expects
            input: Vec<u8, U128>,

            // if omitted `priority` is assumed to be `1`
            // priority: 1,
        },
    },
}

// omitted: `init` and `idle`

fn usart1(ctxt: usart1::Context) {
    const FRAME_DELIMITER: u8 = b'\n';

    let t = &mut ctxt.threshold;
    let tasks = ctxt.tasks;

    let buffer: &mut _ = ctxt.resources.BUFFER;
    let serial: &mut _ = ctxt.resources.SERIAL;

    let byte = serial.read(); // reads a single byte from the serial interface

    if byte == FRAME_DELIMITER {
        tasks.process.schedule_now(t, buffer.clone()).unwrap();
    } else {
        if buffer.push(byte).is_err() {
            // omitted: error handling
        }
    }
}

fn process(ctxt: process::Context) {
    let buffer = ctxt.input;

    match &buffer[..] {
         "command1" => /* .. */,
         "command2" => /* .. */,
         // ..
         _ => /* .. */,
    }
}
```

Here we have the `exti0` task scheduling the `process` task. The `process` task expects some input;
the second argument of `schedule_now` is the expected input. This argument will be sent as a message
to the `process` task.

Only types that implement the `Send` trait and have a `'static` lifetimes can be sent as messages.
This means that messages can't contain references to things like values allocated on the stack of
the task or references to the state of a task.

This constrain forces us to sent a copy of the buffer, which is 128 bytes in size, rather than a
reference, which is 4 bytes in size -- this is rather expensive in terms of memory and execution
time. In a future section we'll see how to make messages much smaller using object pools.

## How is this different from a function call?

You may be wondering how is message passing different that doing a simple function call as shown
below:

``` rust
fn usart1(ctxt: usart1::Context) {
    const FRAME_DELIMITER: u8 = b'\n';

    let buffer: &mut _ = ctxt.resources.BUFFER;
    let serial: &mut _ = ctxt.resources.SERIAL;

    let byte = serial.read(); // reads a single byte from the serial interface

    if byte == FRAME_DELIMITER {
        process(buffer);
    } else {
        if buffer.push(byte).is_err() {
            // omitted: error handling
        }
    }
}

fn process(buffer: &Vec<u8, U128>) {
    match &buffer[..] {
         "command1" => /* .. */,
         "command2" => /* .. */,
         // ..
         _ => /* .. */,
    }
}
```

The function call approach even avoids the expensive copy of the buffer!

The main difference is that a function call will execute `process` in the *same* execution context
as the `usart1` task extending the execution time of the `usart1` task. Whereas making `process`
into its own task means that it can be scheduled differently.

In this particular case the `process` task has lower priority than the `usart1` task so it won't be
executed until *after* the `usart1` task ends. Also, preemption is possible: if a `USART1` event
occurs while executing the `process` task the scheduler will prioritize the execution of the
`usart1` task. The next section has more details about priority based scheduling.
