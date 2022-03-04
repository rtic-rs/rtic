//! examples/task.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use examples_runner as _;

#[rtic::app(device = examples_runner::pac, dispatchers = [SSI0, QEI0])]
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

    #[task]
    fn foo(_: foo::Context) {
        println!("foo - start");

        // spawns `bar` onto the task scheduler
        // `foo` and `bar` have the same priority so `bar` will not run until
        // after `foo` terminates
        bar::spawn().unwrap();

        println!("foo - middle");

        // spawns `baz` onto the task scheduler
        // `baz` has higher priority than `foo` so it immediately preempts `foo`
        baz::spawn().unwrap();

        println!("foo - end");
    }

    #[task]
    fn bar(_: bar::Context) {
        println!("bar");

        exit();
    }

    #[task(priority = 2)]
    fn baz(_: baz::Context) {
        println!("baz");
    }
}
