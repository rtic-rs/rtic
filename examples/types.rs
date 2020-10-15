//! examples/types.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
mod app {
    use cortex_m_semihosting::debug;
    use rtic::cyccnt;

    #[resources]
    struct Resources {
        #[init(0)]
        shared: u32,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        let _: cyccnt::Instant = cx.start;
        let _: rtic::Peripherals = cx.core;
        let _: lm3s6965::Peripherals = cx.device;

        debug::exit(debug::EXIT_SUCCESS);

        init::LateResources {}
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
        let _: &mut u32 = cx.resources.shared;
        let _: foo::Resources = cx.resources;
    }

    // RTIC requires that unused interrupts are declared in an extern block when
    // using software tasks; these free interrupts will be used to dispatch the
    // software tasks.
    extern "C" {
        fn SSI0();
    }
}
