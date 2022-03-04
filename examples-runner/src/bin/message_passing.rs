//! examples/message_passing.rs

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
        foo::spawn(1, 1).unwrap();
        foo::spawn(1, 2).unwrap();
        foo::spawn(2, 3).unwrap();
        assert!(foo::spawn(1, 4).is_err()); // The capacity of `foo` is reached

        (Shared {}, Local {}, init::Monotonics())
    }

    #[task(capacity = 3)]
    fn foo(_c: foo::Context, x: i32, y: u32) {
        println!("foo {}, {}", x, y);
        if x == 2 {
            exit();
        }
    }
}
