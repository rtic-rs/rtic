//! examples/extern_spawn.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use examples_runner::{println, exit};
use examples_runner as _;

// Free function implementing the spawnable task `foo`.
fn foo(_c: app::foo::Context, x: i32, y: u32) {
    println!("foo {}, {}", x, y);
    if x == 2 {
        exit();
    }
    app::foo::spawn(2, 3).unwrap();
}

#[rtic::app(device = examples_runner::pac, dispatchers = [SSI0])]
mod app {
    use crate::foo;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        foo::spawn(1, 2).unwrap();

        (Shared {}, Local {}, init::Monotonics())
    }

    extern "Rust" {
        #[task()]
        fn foo(_c: foo::Context, _x: i32, _y: u32);
    }
}
