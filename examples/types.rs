//! examples/types.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, peripherals = true, dispatchers = [SSI0])]
mod app {
    use cortex_m_semihosting::debug;
    use dwt_systick_monotonic::DwtSystick;

    #[monotonic(binds = SysTick, default = true)]
    type MyMono = DwtSystick<8_000_000>; // 8 MHz

    #[resources]
    struct Resources {
        #[init(0)]
        shared: u32,
    }

    #[init]
    fn init(cx: init::Context) -> (init::LateResources, init::Monotonics) {
        let _: cortex_m::Peripherals = cx.core;
        let _: lm3s6965::Peripherals = cx.device;

        debug::exit(debug::EXIT_SUCCESS);

        let mut dcb = cx.core.DCB;
        let dwt = cx.core.DWT;
        let systick = cx.core.SYST;

        let mono = DwtSystick::new(&mut dcb, dwt, systick, 8_000_000);

        (init::LateResources {}, init::Monotonics(mono))
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }

    #[task(binds = UART0, resources = [shared])]
    fn uart0(cx: uart0::Context) {
        let _: resources::shared = cx.resources.shared;
    }

    #[task(priority = 2, resources = [shared])]
    fn foo(cx: foo::Context) {
        let _: resources::shared = cx.resources.shared;
        let _: foo::Resources = cx.resources;
    }
}
