# Object pools

Let's revisit the message passing example from a few sections ago and make it more efficient using
object pools.

`heapless` provides an object pool abstraction named `Pool` that uses *singleton* buffers. A
singleton buffer is statically allocated and represented by a singleton type, a type of which can
only ever exist one instance of. Normally, `Pool` is `unsafe` to use because the user has to enforce
the singleton requirement of the buffer. RTFM makes `Pool` safe by enforcing the singleton property
of buffers. RTFM accomplishes this by turning all uninitialized resources of array type assigned to
`init` into singleton buffers.

``` rust
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm;
extern crate heapless;

use cortex_m_rtfm::app;
use heapless::Vec;
use heapless::consts::*;
use heapless::pool::{Object, Pool, Uninit};

app! {
    device: stm32f103xx,

    resources: {
        static BUFFER: Option<Object<A>> = None;

        // memory for the `POOL`
        static V: [Vec<u8, U128>; 2];
        static POOL: Pool<V>;
        // ..
    },

    init: {
        resources: [V],
    },

    tasks: {
        usart1: {
            interrupt: USART1,

            priority: 2,

            resources: [BUFFER, POOL, SERIAL],
        },

        process: {
            input: Object<V>,

            // priority: 1,

            // `POOL` is shared with the `usart1` task
            resources: [POOL],
        },
    },
}

fn init(ctxt: init::Context) -> init::LateResources {
    // ..

    let v: Uninit<V> = ctxt.resources.V;

    init::LateResources {
        POOL: Pool::new(v),
    }
}

fn usart1(ctxt: usart1::Context) {
    const FRAME_DELIMITER: u8 = b'\n';

    let t = &mut ctxt.threshold;
    let tasks = ctxt.tasks;

    let rbuffer: &mut _ = ctxt.resources.BUFFER;
    let pool: &mut _ = ctxt.resources.POOL.borrow_mut(t);
    let serial: &mut _ = ctxt.resources.SERIAL;

    if rbuffer.is_none() {
        // grab a buffer from the pool
        *rbuffer = Some(pool.alloc().unwrap().init(Vec::new()));
    }

    let buffer = rbuffer.take().unwrap();

    let byte = serial.read();

    if byte == FRAME_DELIMITER {
        // send the buffer to the `process` task
        tasks.process.schedule_now(t, buffer).unwrap();
    } else {
        if buffer.push(byte).is_err() {
            // omitted: error handling
        }

        rbuffer = Some(buffer);
    }
}

fn process(ctxt: process::Context) {
    let buffer = ctxt.input;

    // process buffer
    match &buffer[..] {
         "command1" => /* .. */,
         "command2" => /* .. */,
         // ..
         _ => /* .. */,
    }

    // return the buffer to the pool
    let t = &mut ctxt.threshold;
    ctxt.resources.POOL.claim_mut(t, |pool, _| pool.dealloc(buffer));
}
```

In this new version we use an object `Pool` that contains two instances of `Vec<u8, U128>`. The
`usart1` task will fill one of the vectors in the `Pool` with data until it finds the frame
delimiter. Once a frame is completed it will send the frame as an `Object` to the `process` task.
Unlike the previous version, the `Object` value is very cheap to send (move): it's just a single
byte in size. In the next iteration `usart1` will grab a fresh, different vector from the `Pool` and
repeat the process.

Once the `process` task is done processing the buffer it will proceed to return it to the object
`Pool`.
