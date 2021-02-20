//! examples/types.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, peripherals = true, dispatchers = [SSI0])]
mod app {
    use cortex_m_semihosting::debug;

    #[resources]
    struct Resources {
        #[init(0)]
        shared: u32,
    }

    #[init]
    fn init(cx: init::Context) -> (init::LateResources, init::Monotonics) {
        let _: cyccnt::Instant = cx.start;
        let _: rtic::Peripherals = cx.core;
        let _: lm3s6965::Peripherals = cx.device;

        debug::exit(debug::EXIT_SUCCESS);

        (init::LateResources {}, init::Monotonics())
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }

    #[task(binds = UART0, resources = [shared])]
    fn uart0(cx: uart0::Context) {
        let _: cyccnt::Instant = cx.start;
        let _: resources::shared = cx.resources.shared;
    }

    #[task(priority = 2, resources = [shared])]
    fn foo(cx: foo::Context) {
        let _: cyccnt::Instant = cx.scheduled;
        let _: resources::shared = cx.resources.shared;
        let _: foo::Resources = cx.resources;
    }
}
