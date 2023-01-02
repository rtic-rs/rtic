//! examples/declared_locals.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [UART0])]
mod app {
    use cortex_m_semihosting::debug;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init(local = [a: u32 = 0])]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        // Locals in `#[init]` have 'static lifetime
        let _a: &'static mut u32 = cx.local.a;

        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator

        (Shared {}, Local {}, init::Monotonics())
    }

    #[idle(local = [a: u32 = 0])]
    fn idle(cx: idle::Context) -> ! {
        // Locals in `#[idle]` have 'static lifetime
        let _a: &'static mut u32 = cx.local.a;

        loop {}
    }

    #[task(local = [a: u32 = 0])]
    fn foo(cx: foo::Context) {
        // Locals in `#[task]`s have a local lifetime
        let _a: &mut u32 = cx.local.a;

        // error: explicit lifetime required in the type of `cx`
        // let _a: &'static mut u32 = cx.local.a;
    }
}
