//! examples/late.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
mod app {
    use cortex_m_semihosting::{debug, hprintln};
    use heapless::{
        consts::*,
        i,
        spsc::{Consumer, Producer, Queue},
    };
    use lm3s6965::Interrupt;

    // Late resources
    #[resources]
    struct Resources {
        p: Producer<'static, u32, U4>,
        c: Consumer<'static, u32, U4>,
        #[task_local]
        #[init(Queue(i::Queue::new()))]
        q: Queue<u32, U4>,
    }

    #[init(resources = [q])]
    fn init(cx: init::Context) -> (init::LateResources, init::Monotonics) {
        // static mut Q: Queue<u32, U4> = Queue(i::Queue::new());

        let (p, c) = cx.resources.q.split();

        // Initialization of late resources
        (init::LateResources { p, c }, init::Monotonics())
    }

    #[idle(resources = [c])]
    fn idle(mut c: idle::Context) -> ! {
        loop {
            if let Some(byte) = c.resources.c.lock(|c| c.dequeue()) {
                hprintln!("received message: {}", byte).unwrap();

                debug::exit(debug::EXIT_SUCCESS);
            } else {
                rtic::pend(Interrupt::UART0);
            }
        }
    }

    #[task(binds = UART0, resources = [p])]
    fn uart0(mut c: uart0::Context) {
        c.resources.p.lock(|p| p.enqueue(42).unwrap());
    }
}
