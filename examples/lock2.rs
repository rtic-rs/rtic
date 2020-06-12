//! examples/lock2.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m_semihosting::debug;
use lm3s6965::Interrupt;

#[rtic::app(device = lm3s6965)]
const APP: () = {
    struct Resources {
        #[init(0)]
        shared: u32,
        #[init(0)]
        shared2: u32,
    }

    #[init]
    fn init(_: init::Context) {
        rtic::pend(Interrupt::GPIOA);
    }

    // when omitted priority is assumed to be `1`
    #[task(binds = GPIOA, resources = [shared, shared2])]
    fn gpioa(mut c: gpioa::Context) {
        c.resources.shared.lock(|shared| {
            *shared += 1;
            rtic::pend(Interrupt::GPIOB);
        });

        debug::exit(debug::EXIT_SUCCESS);
    }

    #[task(binds = GPIOB, priority = 2, resources = [shared, shared2])]
    fn gpiob(c: gpiob::Context) {
        *c.resources.shared += 1;
    }
};
