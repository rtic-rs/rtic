//! examples/late.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
mod app {
    use cortex_m_semihosting::{debug, hprintln};
    use heapless::spsc::{Consumer, Producer, Queue};
    use lm3s6965::Interrupt;

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
                hprintln!("received message: {}", byte).unwrap();

                debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
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
