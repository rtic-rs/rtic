//! examples/preempt.rs

#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![deny(missing_docs)]

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
    fn init(_: init::Context) -> (Shared, Local) {
        foo::spawn().unwrap();

        (Shared {}, Local {})
    }

    #[task(priority = 1)]
    async fn foo(_: foo::Context) {
        hprintln!("foo - start");
        baz::spawn().unwrap();
        hprintln!("foo - end");
        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }

    #[task(priority = 2)]
    async fn bar(_: bar::Context) {
        hprintln!(" bar");
    }

    #[task(priority = 2)]
    async fn baz(_: baz::Context) {
        hprintln!(" baz - start");
        bar::spawn().unwrap();
        hprintln!(" baz - end");
    }
}
