//! examples/lock.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use examples_runner as _;

#[rtic::app(device = examples_runner::pac, dispatchers = [GPIOA, GPIOB, GPIOC])]
mod app {
    use examples_runner::{println, exit};

    #[shared]
    struct Shared {
        shared: u32,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        foo::spawn().unwrap();

        (Shared { shared: 0 }, Local {}, init::Monotonics())
    }

    // when omitted priority is assumed to be `1`
    #[task(shared = [shared])]
    fn foo(mut c: foo::Context) {
        println!("A");

        // the lower priority task requires a critical section to access the data
        c.shared.shared.lock(|shared| {
            // data can only be modified within this critical section (closure)
            *shared += 1;

            // bar will *not* run right now due to the critical section
            bar::spawn().unwrap();

            println!("B - shared = {}", *shared);

            // baz does not contend for `shared` so it's allowed to run now
            baz::spawn().unwrap();
        });

        // critical section is over: bar can now start

        println!("E");

        exit();
    }

    #[task(priority = 2, shared = [shared])]
    fn bar(mut c: bar::Context) {
        // the higher priority task does still need a critical section
        let shared = c.shared.shared.lock(|shared| {
            *shared += 1;

            *shared
        });

        println!("D - shared = {}", shared);
    }

    #[task(priority = 3)]
    fn baz(_: baz::Context) {
        println!("C");
    }
}
