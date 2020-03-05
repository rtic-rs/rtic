//! examples/local.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m_semihosting::{debug, hprintln};
use lm3s6965::Interrupt;
use panic_semihosting as _;

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    struct Resources {
        // An early resource
        #[init(0)]
        shared: u32,

        // A local (move), early resource
        #[task_local]
        #[init(1)]
        l1: u32,

        // An exclusive, early resource
        #[lock_free]
        #[init(1)]
        e1: u32,

        // A local (move), late resource
        #[task_local]
        l2: u32,

        // An exclusive, late resource
        #[lock_free]
        e2: u32,
    }

    #[init]
    fn init(_: init::Context) -> init::LateResources {
        rtfm::pend(Interrupt::UART0);
        rtfm::pend(Interrupt::UART1);
        init::LateResources { e2: 2, l2: 2 }
    }

    // `shared` cannot be accessed from this context
    #[idle(resources =[l1, l2, e2])]
    fn idle(cx: idle::Context) -> ! {
        hprintln!("IDLE:l1 = {}", cx.resources.l1).unwrap();
        hprintln!("IDLE:e2 = {}", cx.resources.e2).unwrap();
        debug::exit(debug::EXIT_SUCCESS);
        loop {}
    }

    // `shared` can be accessed from this context
    #[task(priority = 1, binds = UART0, resources = [shared, l2, e1])]
    fn uart0(cx: uart0::Context) {
        let shared: &mut u32 = cx.resources.shared;
        *shared += 1;
        *cx.resources.e1 += 10;
        hprintln!("UART0: shared = {}", shared).unwrap();
        hprintln!("UART0:l2 = {}", cx.resources.l2).unwrap();
        hprintln!("UART0:e1 = {}", cx.resources.e1).unwrap();
    }

    // l2 should be rejected (not implemented)
    #[task(priority = 2, binds = UART1, resources = [shared, l2, e1])]
    fn uart1(cx: uart1::Context) {
        let shared: &mut u32 = cx.resources.shared;
        *shared += 1;

        hprintln!("UART1: shared = {}", shared).unwrap();
        hprintln!("UART1:l2 = {}", cx.resources.l2).unwrap();
        hprintln!("UART1:e1 = {}", cx.resources.e1).unwrap();
    }

    // if priority is changed we should report a better error message
    // currently, we get an error since RTFM detects a potential race
};
