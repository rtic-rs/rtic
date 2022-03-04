//! examples/destructure.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use examples_runner as _;

#[rtic::app(device = examples_runner::pac, dispatchers = [UART0])]
mod app {
    use examples_runner::{println, exit};

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
        exit();
        // loop {}
    }

    // Direct destructure
    #[task(shared = [&a, &b, &c])]
    fn foo(cx: foo::Context) {
        let a = cx.shared.a;
        let b = cx.shared.b;
        let c = cx.shared.c;

        println!("foo: a = {}, b = {}, c = {}", a, b, c);
    }

    // De-structure-ing syntax
    #[task(shared = [&a, &b, &c])]
    fn bar(cx: bar::Context) {
        let bar::SharedResources { a, b, c } = cx.shared;

        println!("bar: a = {}, b = {}, c = {}", a, b, c);
    }
}
