//! examples/static.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
mod app {
    use cortex_m_semihosting::{debug, hprintln};
    use lm3s6965::Interrupt;

    #[shared]
    struct Shared {
        key: u32,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        rtic::pend(Interrupt::UART0);
        rtic::pend(Interrupt::UART1);

        (Shared { key: 0xdeadbeef }, Local {}, init::Monotonics())
    }

    #[task(binds = UART0, shared = [&key])]
    fn uart0(cx: uart0::Context) {
        let key: &u32 = cx.shared.key;
        hprintln!("UART0(key = {:#x})", key).unwrap();

        debug::exit(debug::EXIT_SUCCESS);
    }

    #[task(binds = UART1, priority = 2, shared = [&key])]
    fn uart1(cx: uart1::Context) {
        hprintln!("UART1(key = {:#x})", cx.shared.key).unwrap();
    }
}
