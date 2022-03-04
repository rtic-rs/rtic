//! examples/only-shared-access.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use examples_runner as _;

#[rtic::app(device = examples_runner::pac, dispatchers = [UART0, UART1])]
mod app {
    use examples_runner::{exit, println};

    #[shared]
    struct Shared {
        key: u32,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        foo::spawn().unwrap();
        bar::spawn().unwrap();

        (Shared { key: 0xdeadbeef }, Local {}, init::Monotonics())
    }

    #[task(shared = [&key])]
    fn foo(cx: foo::Context) {
        let key: &u32 = cx.shared.key;
        println!("foo(key = {:#x})", key);

        exit();
    }

    #[task(priority = 2, shared = [&key])]
    fn bar(cx: bar::Context) {
        println!("bar(key = {:#x})", cx.shared.key);
    }
}
