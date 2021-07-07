//! examples/destructure.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
mod app {
    use cortex_m_semihosting::hprintln;
    use lm3s6965::Interrupt;

    #[shared]
    struct Shared {
        // Some resources to work with
        a: u32,
        b: u32,
        c: u32,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        rtic::pend(Interrupt::UART0);
        rtic::pend(Interrupt::UART1);

        (Shared { a: 0, b: 0, c: 0 }, Local {}, init::Monotonics())
    }

    // Direct destructure
    #[task(binds = UART0, shared = [&a, &b, &c])]
    fn uart0(cx: uart0::Context) {
        let a = cx.shared.a;
        let b = cx.shared.b;
        let c = cx.shared.c;

        hprintln!("UART0: a = {}, b = {}, c = {}", a, b, c).unwrap();
    }

    // De-structure-ing syntax
    #[task(binds = UART1, shared = [&a, &b, &c])]
    fn uart1(cx: uart1::Context) {
        let uart1::SharedResources { a, b, c } = cx.shared;

        hprintln!("UART0: a = {}, b = {}, c = {}", a, b, c).unwrap();
    }
}
