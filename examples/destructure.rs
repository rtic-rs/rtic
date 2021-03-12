//! examples/destructure.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
mod app {
    use cortex_m_semihosting::{debug, hprintln};
    use lm3s6965::Interrupt;

    #[resources]
    struct Resources {
        // Some resources to work with
        #[init(0)]
        a: u32,
        #[init(0)]
        b: u32,
        #[init(0)]
        c: u32,
    }

    #[init]
    fn init(_: init::Context) -> (init::LateResources, init::Monotonics) {
        rtic::pend(Interrupt::UART0);
        rtic::pend(Interrupt::UART1);

        (init::LateResources {}, init::Monotonics())
    }

    #[idle()]
    fn idle(_cx: idle::Context) -> ! {
        debug::exit(debug::EXIT_SUCCESS);
        loop {
            cortex_m::asm::nop();
        }
    }

    // Direct destructure
    #[task(binds = UART0, resources = [&a, &b, &c])]
    fn uart0(cx: uart0::Context) {
        let a = cx.resources.a;
        let b = cx.resources.b;
        let c = cx.resources.c;

        hprintln!("UART0: a = {}, b = {}, c = {}", a, b, c).unwrap();
    }

    // De-structure-ing syntax
    #[task(binds = UART1, resources = [&a, &b, &c])]
    fn uart1(cx: uart1::Context) {
        let uart1::Resources { a, b, c } = cx.resources;

        hprintln!("UART0: a = {}, b = {}, c = {}", a, b, c).unwrap();
    }
}
