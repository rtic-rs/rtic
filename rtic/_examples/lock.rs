//! examples/lock.rs

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]
#![feature(type_alias_impl_trait)]

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
    fn init(_: init::Context) -> (Shared, Local) {
        foo::spawn().unwrap();

        (Shared { shared: 0 }, Local {})
    }

    // when omitted priority is assumed to be `1`
    #[task(shared = [shared])]
    async fn foo(mut c: foo::Context) {
        hprintln!("A");

        // the lower priority task requires a critical section to access the data
        c.shared.shared.lock(|shared| {
            // data can only be modified within this critical section (closure)
            *shared += 1;

            // bar will *not* run right now due to the critical section
            bar::spawn().unwrap();

            hprintln!("B - shared = {}", *shared);

            // baz does not contend for `shared` so it's allowed to run now
            baz::spawn().unwrap();
        });

        // critical section is over: bar can now start

        hprintln!("E");

        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }

    #[task(priority = 2, shared = [shared])]
    async fn bar(mut c: bar::Context) {
        // the higher priority task does still need a critical section
        let shared = c.shared.shared.lock(|shared| {
            *shared += 1;

            *shared
        });

        hprintln!("D - shared = {}", shared);
    }

    #[task(priority = 3)]
    async fn baz(_: baz::Context) {
        hprintln!("C");
    }
}
