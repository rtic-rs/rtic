//! [compile-pass] check that `#[cfg]` attributes are respected

#![no_main]
#![no_std]

use panic_halt as _;

#[rtfm::app(device = lm3s6965, monotonic = rtfm::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        #[cfg(never)]
        #[init(0)]
        foo: u32,
    }

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

    #[task(resources = [foo], schedule = [quux], spawn = [quux])]
    fn foo(_: foo::Context) {
        #[cfg(never)]
        static mut BAR: u32 = 0;
    }

    #[task(priority = 3, resources = [foo], schedule = [quux], spawn = [quux])]
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
