//! examples/message.rs

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
        foo::spawn(/* no message */).unwrap();

        (Shared {}, Local {}, init::Monotonics())
    }

    #[task(local = [count: u32 = 0])]
    fn foo(cx: foo::Context) {
        println!("foo");

        bar::spawn(*cx.local.count).unwrap();
        *cx.local.count += 1;
    }

    #[task]
    fn bar(_: bar::Context, x: u32) {
        println!("bar({})", x);

        baz::spawn(x + 1, x + 2).unwrap();
    }

    #[task]
    fn baz(_: baz::Context, x: u32, y: u32) {
        println!("baz({}, {})", x, y);

        if x + y > 4 {
            exit();
        }

        foo::spawn().unwrap();
    }
}
