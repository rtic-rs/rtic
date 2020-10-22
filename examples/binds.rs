//! examples/binds.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

// `examples/interrupt.rs` rewritten to use `binds`
#[rtic::app(device = lm3s6965)]
mod app {
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

    #[task(binds = UART0)]
    fn foo(_: foo::Context) {
        static mut TIMES: u32 = 0;

        *TIMES += 1;

        hprintln!(
            "foo called {} time{}",
            *TIMES,
            if *TIMES > 1 { "s" } else { "" }
        )
        .unwrap();
    }
}
