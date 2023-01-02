//! examples/lock.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [GPIOA, GPIOB, GPIOC])]
mod app {
    use cortex_m_semihosting::{debug, hprintln};

    #[shared]
    struct Shared {
        shared: u32,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        foo::spawn().unwrap();

        (Shared { shared: 0 }, Local {}, init::Monotonics())
    }

    // when omitted priority is assumed to be `1`
    #[task(shared = [shared])]
    fn foo(mut c: foo::Context) {
        hprintln!("A").unwrap();

        // the lower priority task requires a critical section to access the data
        c.shared.shared.lock(|shared| {
            // data can only be modified within this critical section (closure)
            *shared += 1;

            // bar will *not* run right now due to the critical section
            bar::spawn().unwrap();

            hprintln!("B - shared = {}", *shared).unwrap();

            // baz does not contend for `shared` so it's allowed to run now
            baz::spawn().unwrap();
        });

        // critical section is over: bar can now start

        hprintln!("E").unwrap();

        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }

    #[task(priority = 2, shared = [shared])]
    fn bar(mut c: bar::Context) {
        // the higher priority task does still need a critical section
        let shared = c.shared.shared.lock(|shared| {
            *shared += 1;

            *shared
        });

        hprintln!("D - shared = {}", shared).unwrap();
    }

    #[task(priority = 3)]
    fn baz(_: baz::Context) {
        hprintln!("C").unwrap();
    }
}
