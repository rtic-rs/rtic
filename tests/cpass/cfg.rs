//! Compile-pass test that checks that `#[cfg]` attributes are respected

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate lm3s6965;
extern crate panic_semihosting;
extern crate rtfm;

use rtfm::app;

#[app(device = lm3s6965)]
const APP: () = {
    #[cfg(never)]
    static mut FOO: u32 = 0;

    #[init]
    fn init() {
        #[cfg(never)]
        static mut BAR: u32 = 0;
    }

    #[idle]
    fn idle() -> ! {
        #[cfg(never)]
        static mut BAR: u32 = 0;

        loop {}
    }

    #[task(resources = [FOO], schedule = [quux], spawn = [quux])]
    fn foo() {
        #[cfg(never)]
        static mut BAR: u32 = 0;
    }

    #[task(priority = 3, resources = [FOO], schedule = [quux], spawn = [quux])]
    fn bar() {
        #[cfg(never)]
        static mut BAR: u32 = 0;
    }

    #[cfg(never)]
    #[task]
    fn quux() {}

    extern "C" {
        fn UART0();
        fn UART1();
    }
};
