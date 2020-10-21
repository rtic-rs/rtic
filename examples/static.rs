//! examples/static.rs

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
        ppppp: Producer<'static, u32, U4>,
        c: Consumer<'static, u32, U4>,
    }

    #[init]
    fn init(_: init::Context) -> init::LateResources {
        static mut Q: Queue<u32, U4> = Queue(i::Queue::new());

        let (ppppp, c) = Q.split();

        // Initialization of late resources
        init::LateResources { ppppp, c }
    }

    #[idle(resources = [c])]
    fn idle(c: idle::Context) -> ! {
        loop {
            if let Some(byte) = c.resources.c.dequeue() {
                hprintln!("received message: {}", byte).unwrap();

                debug::exit(debug::EXIT_SUCCESS);
            } else {
                rtic::pend(Interrupt::UART0);
            }
        }
    }

    #[task(binds = UART0, resources = [ppppp])]
    fn uart0(c: uart0::Context) {
        static mut KALLE: u32 = 0;
        *KALLE += 1;
        c.resources.ppppp.enqueue(42).unwrap();
    }
}
