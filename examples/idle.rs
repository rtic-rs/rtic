//! examples/idle.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_semihosting;

use cortex_m_semihosting::debug;
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
        println!("init");
    }

    #[idle]
    fn idle() -> ! {
        static mut X: u32 = 0;

        // Safe access to local `static mut` variable
        let _x: &'static mut u32 = X;

        println!("idle");

        debug::exit(debug::EXIT_SUCCESS);

        loop {}
    }
};
