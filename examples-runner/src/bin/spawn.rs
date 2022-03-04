//! examples/spawn.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use examples_runner as _;

#[rtic::app(device = examples_runner::pac, dispatchers = [SSI0])]
mod app {
    use examples_runner::{println, exit};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        println!("init");
        foo::spawn().unwrap();

        (Shared {}, Local {}, init::Monotonics())
    }

    #[task]
    fn foo(_: foo::Context) {
        println!("foo");

        exit();
    }
}
