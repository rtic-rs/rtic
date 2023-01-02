//! examples/destructure.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [UART0])]
mod app {
    use cortex_m_semihosting::{debug, hprintln};

    #[shared]
    struct Shared {
        a: u32,
        b: u32,
        c: u32,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        foo::spawn().unwrap();
        bar::spawn().unwrap();

        (Shared { a: 0, b: 0, c: 0 }, Local {}, init::Monotonics())
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
        loop {}
    }

    // Direct destructure
    #[task(shared = [&a, &b, &c])]
    fn foo(cx: foo::Context) {
        let a = cx.shared.a;
        let b = cx.shared.b;
        let c = cx.shared.c;

        hprintln!("foo: a = {}, b = {}, c = {}", a, b, c).unwrap();
    }

    // De-structure-ing syntax
    #[task(shared = [&a, &b, &c])]
    fn bar(cx: bar::Context) {
        let bar::SharedResources { a, b, c } = cx.shared;

        hprintln!("bar: a = {}, b = {}, c = {}", a, b, c).unwrap();
    }
}
