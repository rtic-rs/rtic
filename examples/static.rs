//! examples/static.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [UART0])]
mod app {
    use cortex_m_semihosting::{debug, hprintln};
    use heapless::spsc::{Consumer, Producer, Queue};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        p: Producer<'static, u32, 5>,
        c: Consumer<'static, u32, 5>,
    }

    #[init(local = [q: Queue<u32, 5> = Queue::new()])]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        // q has 'static life-time so after the split and return of `init`
        // it will continue to exist and be allocated
        let (p, c) = cx.local.q.split();

        foo::spawn().unwrap();

        (Shared {}, Local { p, c }, init::Monotonics())
    }

    #[idle(local = [c])]
    fn idle(c: idle::Context) -> ! {
        loop {
            // Lock-free access to the same underlying queue!
            if let Some(data) = c.local.c.dequeue() {
                hprintln!("received message: {}", data).unwrap();

                // Run foo until data
                if data == 3 {
                    debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
                } else {
                    foo::spawn().unwrap();
                }
            }
        }
    }

    #[task(local = [p, state: u32 = 0])]
    fn foo(c: foo::Context) {
        *c.local.state += 1;

        // Lock-free access to the same underlying queue!
        c.local.p.enqueue(*c.local.state).unwrap();
    }
}
