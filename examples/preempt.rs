//! examples/preempt.rs

#![no_main]
#![no_std]

use panic_semihosting as _;
use rtic::app;

#[app(device = lm3s6965, dispatchers = [SSI0, QEI0])]
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

    #[task(priority = 1)]
    fn foo(_: foo::Context) {
        hprintln!("foo - start");
        baz::spawn().unwrap();
        hprintln!("foo - end");
        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }

    #[task(priority = 2)]
    fn bar(_: bar::Context) {
        hprintln!(" bar");
    }

    #[task(priority = 2)]
    fn baz(_: baz::Context) {
        hprintln!(" baz - start");
        bar::spawn().unwrap();
        hprintln!(" baz - end");
    }
}
