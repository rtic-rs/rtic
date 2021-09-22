//! examples/task.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0, QEI0])]
mod app {
    use cortex_m_semihosting::{debug, hprintln};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        foo::spawn().unwrap();

        (Shared {}, Local {}, init::Monotonics())
    }

    #[task]
    fn foo(_: foo::Context) {
        hprintln!("foo - start").unwrap();

        // spawns `bar` onto the task scheduler
        // `foo` and `bar` have the same priority so `bar` will not run until
        // after `foo` terminates
        bar::spawn().unwrap();

        hprintln!("foo - middle").unwrap();

        // spawns `baz` onto the task scheduler
        // `baz` has higher priority than `foo` so it immediately preempts `foo`
        baz::spawn().unwrap();

        hprintln!("foo - end").unwrap();
    }

    #[task]
    fn bar(_: bar::Context) {
        hprintln!("bar").unwrap();

        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }

    #[task(priority = 2)]
    fn baz(_: baz::Context) {
        hprintln!("baz").unwrap();
    }
}
