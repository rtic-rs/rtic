//! examples/binds.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_semihosting;

use cortex_m_semihosting::{debug, hprintln};
use lm3s6965::Interrupt;
use rtfm::app;

// `examples/interrupt.rs` rewritten to use `binds`
#[app(device = lm3s6965)]
const APP: () = {
    #[init]
    fn init() {
        rtfm::pend(Interrupt::UART0);

        hprintln!("init").unwrap();
    }

    #[idle]
    fn idle() -> ! {
        hprintln!("idle").unwrap();

        rtfm::pend(Interrupt::UART0);

        debug::exit(debug::EXIT_SUCCESS);

        loop {}
    }

    #[interrupt(binds = UART0)]
    fn foo() {
        static mut TIMES: u32 = 0;

        *TIMES += 1;

        hprintln!(
            "foo called {} time{}",
            *TIMES,
            if *TIMES > 1 { "s" } else { "" }
        )
        .unwrap();
    }
};
