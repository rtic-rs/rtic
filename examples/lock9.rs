//! examples/lock.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m_semihosting::{debug, hprintln};
use lm3s6965::Interrupt;
use panic_semihosting as _;
use rtic::Exclusive;

#[rtic::app(device = lm3s6965)]
const APP: () = {
    struct Resources {
        #[init(0)]
        shared: u32,
    }

    #[init]
    fn init(_: init::Context) {
        rtic::pend(Interrupt::GPIOA);
    }

    // when omitted priority is assumed to be `1`
    #[task(binds = GPIOA, resources = [shared])]
    fn gpioa(c: gpioa::Context) {
        hprintln!("A").unwrap();

        // the lower priority task requires a critical section to access the data
        c.resources.shared.lock(|shared| {
            // data can only be modified within this critical section (closure)
            *shared += 1;

            // GPIOB will *not* run right now due to the critical section
            rtic::pend(Interrupt::GPIOB);

            hprintln!("B - shared = {}", *shared).unwrap();

            // GPIOC does not contend for `shared` so it's allowed to run now
            rtic::pend(Interrupt::GPIOC);
        });

        // critical section is over: GPIOB can now start

        hprintln!("E").unwrap();

        debug::exit(debug::EXIT_SUCCESS);
    }

    #[task(binds = GPIOB, priority = 2, resources = [shared])]
    fn gpiob(c: gpiob::Context) {
        c.resources.shared.lock(|shared| {
            *shared += 1;
            hprintln!("D - shared = {}", shared).unwrap();
        });
    }

    #[task(binds = GPIOC, priority = 3, resources = [shared])]
    fn gpioc(c: gpioc::Context) {
        let se1 = Exclusive::new(c.resources.shared);
        let se2 = Exclusive::new(c.resources.shared);
        se1.lock(|s1| {
            se2.lock(|s2| {
                *s1 = 2;
                *s2 = 3;
            });
        });
    }
};
