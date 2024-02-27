//! examples/task.rs

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0, QEI0])]
mod app {
    use cortex_m_semihosting::{debug, hprintln};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        foo::spawn().unwrap();

        (Shared {}, Local {})
    }

    #[task]
    async fn foo(_: foo::Context) {
        hprintln!("foo - start");

        // spawns `bar` onto the task scheduler
        // `foo` and `bar` have the same priority so `bar` will not run until
        // after `foo` terminates
        bar::spawn().unwrap();

        hprintln!("foo - middle");

        // spawns `baz` onto the task scheduler
        // `baz` has higher priority than `foo` so it immediately preempts `foo`
        baz::spawn().unwrap();

        hprintln!("foo - end");
    }

    #[task]
    async fn bar(_: bar::Context) {
        hprintln!("bar");

        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }

    #[task(priority = 2)]
    async fn baz(_: baz::Context) {
        hprintln!("baz");
    }
}
