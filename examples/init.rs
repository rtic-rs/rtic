//! examples/init.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_semihosting;

use cortex_m_semihosting::debug;
use rtfm::app;

// NOTE: This convenience macro will appear in all the other examples and
// will always look the same
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
        static mut X: u32 = 0;

        // Cortex-M peripherals
        let _core: rtfm::Peripherals = core;

        // Device specific peripherals
        let _device: lm3s6965::Peripherals = device;

        // Safe access to local `static mut` variable
        let _x: &'static mut u32 = X;

        println!("init");

        debug::exit(debug::EXIT_SUCCESS);
    }
};
