//! Compile-pass test that checks that `#[cfg]` attributes are respected

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate lm3s6965;
extern crate panic_halt;
extern crate rtfm;

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    #[cfg(never)]
    static mut FOO: u32 = 0;

    #[init]
    fn init(_: init::Context) {
        #[cfg(never)]
        static mut BAR: u32 = 0;
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        #[cfg(never)]
        static mut BAR: u32 = 0;

        loop {}
    }

    #[task(resources = [FOO], schedule = [quux], spawn = [quux])]
    fn foo(_: foo::Context) {
        #[cfg(never)]
        static mut BAR: u32 = 0;
    }

    #[task(priority = 3, resources = [FOO], schedule = [quux], spawn = [quux])]
    fn bar(_: bar::Context) {
        #[cfg(never)]
        static mut BAR: u32 = 0;
    }

    #[cfg(never)]
    #[task]
    fn quux(_: quux::Context) {}

    extern "C" {
        fn UART0();
        fn UART1();
    }
};
