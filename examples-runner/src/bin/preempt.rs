//! examples/preempt.rs

#![no_main]
#![no_std]

use examples_runner as _;
use rtic::app;

#[app(device = examples_runner::pac, dispatchers = [SSI0, QEI0])]
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

    #[task(priority = 1)]
    fn foo(_: foo::Context) {
        println!("foo - start");
        baz::spawn().unwrap();
        println!("foo - end");
        exit();
    }

    #[task(priority = 2)]
    fn bar(_: bar::Context) {
        println!(" bar");
    }

    #[task(priority = 2)]
    fn baz(_: baz::Context) {
        println!(" baz - start");
        bar::spawn().unwrap();
        println!(" baz - end");
    }
}
