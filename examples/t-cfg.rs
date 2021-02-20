//! [compile-pass] check that `#[cfg]` attributes are respected

#![no_main]
#![no_std]

use panic_halt as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0, QEI0])]
mod app {
    #[resources]
    struct Resources {
        #[cfg(never)]
        #[init(0)]
        foo: u32,
    }

    #[init]
    fn init(_: init::Context) -> (init::LateResources, init::Monotonics) {
        #[cfg(never)]
        static mut BAR: u32 = 0;

        (init::LateResources {}, init::Monotonics())
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        #[cfg(never)]
        static mut BAR: u32 = 0;

        loop {
            cortex_m::asm::nop();
        }
    }

    #[task(resources = [foo])]
    fn foo(_: foo::Context) {
        #[cfg(never)]
        static mut BAR: u32 = 0;
    }

    #[task(priority = 3, resources = [foo])]
    fn bar(_: bar::Context) {
        #[cfg(never)]
        static mut BAR: u32 = 0;
    }

    #[cfg(never)]
    #[task]
    fn quux(_: quux::Context) {}
}
