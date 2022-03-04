//! examples/ramfunc.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use examples_runner as _;

#[rtic::app(
    device = examples_runner::pac,
    dispatchers = [
        UART0,
        #[link_section = ".data.UART1"]
        UART1
    ])
]
mod app {
    use examples_runner::{println, exit};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        foo::spawn().unwrap();

        (Shared {}, Local {}, init::Monotonics())
    }

    #[inline(never)]
    #[task]
    fn foo(_: foo::Context) {
        println!("foo");

        exit();
    }

    // run this task from RAM
    #[inline(never)]
    #[link_section = ".data.bar"]
    #[task(priority = 2)]
    fn bar(_: bar::Context) {
        foo::spawn().unwrap();
    }
}
