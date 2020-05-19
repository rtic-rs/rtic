//! [compile-pass] check that `#[cfg]` attributes are respected

#![no_main]
#![no_std]

use panic_halt as _;

#[rtic::app(device = lm3s6965, monotonic = rtic::cyccnt::CYCCNT)]
mod app {
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

        loop {
            cortex_m::asm::nop();
        }
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

    // RTIC requires that unused interrupts are declared in an extern block when
    // using software tasks; these free interrupts will be used to dispatch the
    // software tasks.
    extern "C" {
        fn SSI0();
        fn QEI0();
    }
}
