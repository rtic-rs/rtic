//! examples/late.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use examples_runner as _;

#[rtic::app(device = examples_runner::pac)]
mod app {
    use examples_runner::{println, exit};
    use heapless::spsc::{Consumer, Producer, Queue};
    use examples_runner::pac::Interrupt;

    #[shared]
    struct Shared {
        p: Producer<'static, u32, 5>,
        c: Consumer<'static, u32, 5>,
    }

    #[local]
    struct Local {}

    #[init(local = [q: Queue<u32, 5> = Queue::new()])]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        let (p, c) = cx.local.q.split();

        // Initialization of shared resources
        (Shared { p, c }, Local {}, init::Monotonics())
    }

    #[idle(shared = [c])]
    fn idle(mut c: idle::Context) -> ! {
        loop {
            if let Some(byte) = c.shared.c.lock(|c| c.dequeue()) {
                println!("received message: {}", byte);

                exit();
            } else {
                rtic::pend(Interrupt::UART0);
            }
        }
    }

    #[task(binds = UART0, shared = [p])]
    fn uart0(mut c: uart0::Context) {
        c.shared.p.lock(|p| p.enqueue(42).unwrap());
    }
}
