//! examples/lock-free.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use examples_runner as _;

#[rtic::app(device = examples_runner::pac, dispatchers = [GPIOA])]
mod app {
    use examples_runner::{println, exit};

    #[shared]
    struct Shared {
        #[lock_free] // <- lock-free shared resource
        counter: u64,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        foo::spawn().unwrap();

        (Shared { counter: 0 }, Local {}, init::Monotonics())
    }

    #[task(shared = [counter])] // <- same priority
    fn foo(c: foo::Context) {
        bar::spawn().unwrap();

        *c.shared.counter += 1; // <- no lock API required
        let counter = *c.shared.counter;
        println!("  foo = {}", counter);
    }

    #[task(shared = [counter])] // <- same priority
    fn bar(c: bar::Context) {
        foo::spawn().unwrap();

        *c.shared.counter += 1; // <- no lock API required
        let counter = *c.shared.counter;
        println!("  bar = {}", counter);

        exit();
    }
}
