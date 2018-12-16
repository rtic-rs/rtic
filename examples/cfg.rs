//! examples/cfg.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_semihosting;

#[cfg(debug_assertions)]
use cortex_m_semihosting::hprintln;
use rtfm::app;

#[app(device = lm3s6965)]
const APP: () = {
    #[cfg(debug_assertions)] // <- `true` when using the `dev` profile
    static mut COUNT: u32 = 0;

    #[init]
    fn init() {
        // ..
    }

    #[task(priority = 3, resources = [COUNT], spawn = [log])]
    fn foo() {
        #[cfg(debug_assertions)]
        {
            *resources.COUNT += 1;

            spawn.log(*resources.COUNT).ok();
        }

        // this wouldn't compile in `release` mode
        // *resources.COUNT += 1;

        // ..
    }

    #[cfg(debug_assertions)]
    #[task]
    fn log(n: u32) {
        hprintln!(
            "foo has been called {} time{}",
            n,
            if n == 1 { "" } else { "s" }
        )
        .ok();
    }

    extern "C" {
        fn UART0();
        fn UART1();
    }
};
