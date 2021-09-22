//! examples/ramfunc.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

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
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        foo::spawn().unwrap();

        (Shared {}, Local {}, init::Monotonics())
    }

    #[inline(never)]
    #[task]
    fn foo(_: foo::Context) {
        hprintln!("foo").unwrap();

        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }

    // run this task from RAM
    #[inline(never)]
    #[link_section = ".data.bar"]
    #[task(priority = 2)]
    fn bar(_: bar::Context) {
        foo::spawn().unwrap();
    }
}
