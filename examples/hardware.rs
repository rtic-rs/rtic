//! examples/hardware.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m_semihosting::{debug, hprintln};
use lm3s6965::Interrupt;
use panic_semihosting as _;

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    #[init]
    fn init(_: init::Context) {
        // Pends the UART0 interrupt but its handler won't run until *after*
        // `init` returns because interrupts are disabled
        rtfm::pend(Interrupt::UART0); // equivalent to NVIC::pend

        hprintln!("init").unwrap();
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        // interrupts are enabled again; the `UART0` handler runs at this point

        hprintln!("idle").unwrap();

        rtfm::pend(Interrupt::UART0);

        debug::exit(debug::EXIT_SUCCESS);

        loop {}
    }

    #[task(binds = UART0)]
    fn uart0(_: uart0::Context) {
        static mut TIMES: u32 = 0;

        // Safe access to local `static mut` variable
        *TIMES += 1;

        hprintln!(
            "UART0 called {} time{}",
            *TIMES,
            if *TIMES > 1 { "s" } else { "" }
        )
        .unwrap();
    }
};
