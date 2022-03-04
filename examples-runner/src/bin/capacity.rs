//! examples/capacity.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use examples_runner as _;

#[rtic::app(device = examples_runner::pac, dispatchers = [SSI0])]
mod app {
    use examples_runner::{println, exit};
    use examples_runner::pac::Interrupt;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        rtic::pend(Interrupt::UART0);

        (Shared {}, Local {}, init::Monotonics())
    }

    #[task(binds = UART0)]
    fn uart0(_: uart0::Context) {
        foo::spawn(0).unwrap();
        foo::spawn(1).unwrap();
        foo::spawn(2).unwrap();
        foo::spawn(3).unwrap();

        bar::spawn().unwrap();
    }

    #[task(capacity = 4)]
    fn foo(_: foo::Context, x: u32) {
        println!("foo({})", x);
    }

    #[task]
    fn bar(_: bar::Context) {
        println!("bar");

        exit();
    }
}
