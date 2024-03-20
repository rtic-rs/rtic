//! examples/extern_binds.rs

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]

use cortex_m_semihosting::{debug, hprintln};
use lm3s6965::Interrupt;
use panic_semihosting as _;

// Free function implementing `init`.
fn init(_: app::init::Context) -> (app::Shared, app::Local) {
    rtic::pend(Interrupt::UART0);

    hprintln!("init");

    (app::Shared {}, app::Local {})
}

// Free function implementing `idle`.
fn idle(_: app::idle::Context) -> ! {
    hprintln!("idle");

    rtic::pend(Interrupt::UART0);

    loop {
        cortex_m::asm::nop();
        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }
}

// Free function implementing the interrupt bound task `foo`.
fn foo(_: app::foo::Context) {
    hprintln!("foo called");
}

#[rtic::app(device = lm3s6965)]
mod app {
    use crate::{foo, idle, init};

    #[shared]
    pub struct Shared {}

    #[local]
    pub struct Local {}

    extern "Rust" {
        #[init]
        fn init(_: init::Context) -> (Shared, Local);

        #[idle]
        fn idle(_: idle::Context) -> !;

        #[task(binds = UART0)]
        fn foo(_: foo::Context);
    }
}
