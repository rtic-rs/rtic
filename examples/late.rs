//! examples/late.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m_semihosting::{debug, hprintln};
use heapless::{
    consts::*,
    spsc::{Consumer, Producer, Queue},
};
use lm3s6965::Interrupt;
use panic_semihosting as _;

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    // Late resources
    extern "C" {
        static mut P: Producer<'static, u32, U4>;
        static mut C: Consumer<'static, u32, U4>;
    }

    #[init]
    fn init(_: init::Context) -> init::LateResources {
        // NOTE: we use `Option` here to work around the lack of
        // a stable `const` constructor
        static mut Q: Option<Queue<u32, U4>> = None;

        *Q = Some(Queue::new());
        let (p, c) = Q.as_mut().unwrap().split();

        // Initialization of late resources
        init::LateResources { P: p, C: c }
    }

    #[idle(resources = [C])]
    fn idle(c: idle::Context) -> ! {
        loop {
            if let Some(byte) = c.resources.C.dequeue() {
                hprintln!("received message: {}", byte).unwrap();

                debug::exit(debug::EXIT_SUCCESS);
            } else {
                rtfm::pend(Interrupt::UART0);
            }
        }
    }

    #[task(binds = UART0, resources = [P])]
    fn uart0(c: uart0::Context) {
        c.resources.P.enqueue(42).unwrap();
    }
};
