//! examples/lock-free.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
mod app {
    use cortex_m_semihosting::{debug, hprintln};
    use lm3s6965::Interrupt;

    #[shared]
    struct Shared {
        #[lock_free] // <- lock-free shared resource
        counter: u64,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        rtic::pend(Interrupt::GPIOA);

        (Shared { counter: 0 }, Local {}, init::Monotonics())
    }

    #[task(binds = GPIOA, shared = [counter])] // <- same priority
    fn gpioa(c: gpioa::Context) {
        hprintln!("GPIOA/start").unwrap();
        rtic::pend(Interrupt::GPIOB);

        *c.shared.counter += 1; // <- no lock API required
        let counter = *c.shared.counter;
        hprintln!("  GPIOA/counter = {}", counter).unwrap();

        if counter == 5 {
            debug::exit(debug::EXIT_SUCCESS);
        }
        hprintln!("GPIOA/end").unwrap();
    }

    #[task(binds = GPIOB, shared = [counter])] // <- same priority
    fn gpiob(c: gpiob::Context) {
        hprintln!("GPIOB/start").unwrap();
        rtic::pend(Interrupt::GPIOA);

        *c.shared.counter += 1; // <- no lock API required
        let counter = *c.shared.counter;
        hprintln!("  GPIOB/counter = {}", counter).unwrap();

        if counter == 5 {
            debug::exit(debug::EXIT_SUCCESS);
        }
        hprintln!("GPIOB/end").unwrap();
    }
}
