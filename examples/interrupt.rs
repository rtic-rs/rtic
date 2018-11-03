//! examples/interrupt.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_semihosting;

use cortex_m_semihosting::debug;
use lm3s6965::Interrupt;
use rtfm::app;

macro_rules! println {
    ($($tt:tt)*) => {
        if let Ok(mut stdout) = cortex_m_semihosting::hio::hstdout() {
            use core::fmt::Write;

            writeln!(stdout, $($tt)*).ok();
        }
    };
}

#[app(device = lm3s6965)]
const APP: () = {
    #[init]
    fn init() {
        // Pends the UART0 interrupt but its handler won't run until *after*
        // `init` returns because interrupts are disabled
        rtfm::pend(Interrupt::UART0);

        println!("init");
    }

    #[idle]
    fn idle() -> ! {
        // interrupts are enabled again; the `UART0` handler runs at this point

        println!("idle");

        rtfm::pend(Interrupt::UART0);

        debug::exit(debug::EXIT_SUCCESS);

        loop {}
    }

    #[interrupt]
    fn UART0() {
        static mut TIMES: u32 = 0;

        // Safe access to local `static mut` variable
        *TIMES += 1;

        println!(
            "UART0 called {} time{}",
            *TIMES,
            if *TIMES > 1 { "s" } else { "" }
        );
    }
};
