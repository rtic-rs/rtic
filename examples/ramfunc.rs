//! examples/ramfunc.rs

#![deny(warnings)]
#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
use panic_semihosting as _;

#[rtic::app(
    device = lm3s6965,
    dispatchers = [
        UART0,
        #[link_section = ".data.UART1"]
        UART1
    ])
]
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

    #[inline(never)]
    #[task]
    async fn foo(_: foo::Context) {
        hprintln!("foo").unwrap();

        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }

    // run this task from RAM
    #[inline(never)]
    #[link_section = ".data.bar"]
    #[task(priority = 2)]
    async fn bar(_: bar::Context) {
        foo::spawn().unwrap();
    }
}
