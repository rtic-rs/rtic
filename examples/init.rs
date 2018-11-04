//! examples/init.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_semihosting;

use cortex_m_semihosting::{debug, hprintln};
use rtfm::app;

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

        hprintln!("init").unwrap();

        debug::exit(debug::EXIT_SUCCESS);
    }
};
