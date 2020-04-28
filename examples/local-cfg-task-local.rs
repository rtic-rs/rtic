//! examples/local-cfg-task-local.rs

#![deny(unsafe_code)]
//#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m_semihosting::hprintln;
use cortex_m_semihosting::debug;
use lm3s6965::Interrupt;
use panic_semihosting as _;

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    struct Resources {
        // A local (move), early resource
        #[cfg(feature = "feature_l1")]
        #[task_local]
        #[init(1)]
        l1: u32,

        // A local (move), late resource
        #[task_local]
        l2: u32,
    }

    #[init]
    fn init(_: init::Context) -> init::LateResources {
        rtfm::pend(Interrupt::UART0);
        rtfm::pend(Interrupt::UART1);
        init::LateResources {
            #[cfg(feature = "feature_l2")]
            l2: 2,
            #[cfg(not(feature = "feature_l2"))]
            l2: 5
        }
    }

    // l1 ok (task_local)
    #[idle(resources =[#[cfg(feature = "feature_l1")]l1])]
    fn idle(_cx: idle::Context) -> ! {
        #[cfg(feature = "feature_l1")]
        hprintln!("IDLE:l1 = {}", _cx.resources.l1).unwrap();
        debug::exit(debug::EXIT_SUCCESS);
        loop {}
    }

    // l2 ok (task_local)
    #[task(priority = 1, binds = UART0, resources = [
        #[cfg(feature = "feature_l2")]l2,
    ])]
    fn uart0(_cx: uart0::Context) {
        #[cfg(feature = "feature_l2")]
        hprintln!("UART0:l2 = {}", _cx.resources.l2).unwrap();
    }

    // l2 error, conflicting with uart0 for l2 (task_local)
    #[task(priority = 1, binds = UART1, resources = [
        #[cfg(not(feature = "feature_l2"))]l2
    ])]
    fn uart1(_cx: uart1::Context) {
        #[cfg(not(feature = "feature_l2"))]
        hprintln!("UART0:l2 = {}", _cx.resources.l2).unwrap();
    }
};
