//! zero priority task
#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]

use hifive1::hal::e310x;
use riscv_rt as _;

#[cfg_attr(feature = "riscv-mecall-backend", rtic::app(device = e310x))]
#[cfg_attr(feature = "riscv-clint-backend", rtic::app(device = e310x, backend = H0))]
mod app {
    use super::e310x;
    use heapless::spsc::{Consumer, Producer, Queue};
    use semihosting::{println, process::exit};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        p: Producer<'static, u32, 5>,
        c: Consumer<'static, u32, 5>,
    }

    #[init(local = [q: Queue<u32, 5> = Queue::new()])]
    fn init(cx: init::Context) -> (Shared, Local) {
        // q has 'static life-time so after the split and return of `init`
        // it will continue to exist and be allocated
        let (p, c) = cx.local.q.split();

        foo::spawn().unwrap();

        (Shared {}, Local { p, c })
    }

    #[idle(local = [c])]
    fn idle(c: idle::Context) -> ! {
        loop {
            // Lock-free access to the same underlying queue!
            if let Some(data) = c.local.c.dequeue() {
                println!("received message: {}", data);

                // Run foo until data
                if data == 3 {
                    exit(0); // Exit QEMU simulator
                } else {
                    foo::spawn().unwrap();
                }
            }
        }
    }

    #[task(local = [p, state: u32 = 0], priority = 1)]
    async fn foo(c: foo::Context) {
        *c.local.state += 1;

        // Lock-free access to the same underlying queue!
        c.local.p.enqueue(*c.local.state).unwrap();
    }
}
