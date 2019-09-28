//! examples/optlock.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m_semihosting::debug;
use lm3s6965::Interrupt;
use panic_semihosting as _;
use rtfm::Exclusive;

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    struct Resources {
        #[init(0)]
        shared: u32,
    }

    #[init]
    fn init(_: init::Context) {
        rtfm::pend(Interrupt::GPIOA);
    }

    // when omitted priority is assumed to be `1`
    #[task(binds = GPIOA, resources = [shared])]
    fn gpioa(mut c: gpioa::Context) {
        // the lower priority task requires a critical section to access the data
        c.resources.shared.lock(|shared| {
            // data can only be modified within this critical section (closure)
            *shared += 1;

            // GPIOB will *not* run right now due to the critical section
            rtfm::pend(Interrupt::GPIOB);

            //    hprintln!("B - shared = {}", *shared).unwrap();

            // GPIOC does not contend for `shared` so it's allowed to run now
            rtfm::pend(Interrupt::GPIOC);
        });

        // critical section is over: GPIOB can now start

        debug::exit(debug::EXIT_SUCCESS);
    }

    #[task(binds = GPIOB, priority = 2, resources = [shared])]
    fn gpiob(mut c: gpiob::Context) {
        // higher priority task, with critical section
        c.resources.shared.lock(|shared| {
            *shared += 1;
        });
    }

    #[task(binds = GPIOC, priority = 3, resources = [shared])]
    fn gpioc(c: gpioc::Context) {
        // highest priority task with critical section
        // we wrap resource.shared into an Exclusive
        let mut exclusive = Exclusive(c.resources.shared);
        // we can access through both lock ...
        exclusive.lock(|shared| {
            *shared += 1;
        });
        // and deref, i.e., non-orthogonal design
        *exclusive += 1;
    }
};
