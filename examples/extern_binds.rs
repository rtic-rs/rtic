//! examples/extern_binds.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m_semihosting::hprintln;
use panic_semihosting as _;

// Free function implementing the interrupt bound task `foo`.
fn foo(_: app::foo::Context) {
    hprintln!("foo called").ok();
}

#[rtic::app(device = lm3s6965)]
mod app {
    use crate::foo;
    use cortex_m_semihosting::{debug, hprintln};
    use lm3s6965::Interrupt;

    #[init]
    fn init(_: init::Context) -> init::LateResources {
        rtic::pend(Interrupt::UART0);

        hprintln!("init").unwrap();

        init::LateResources {}
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        hprintln!("idle").unwrap();

        rtic::pend(Interrupt::UART0);

        debug::exit(debug::EXIT_SUCCESS);

        loop {
            cortex_m::asm::nop();
        }
    }

    extern "C" {
        #[task(binds = UART0)]
        fn foo(_: foo::Context);
    }
}
