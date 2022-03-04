//! examples/pool.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use heapless::{
    pool,
    pool::singleton::{Box, Pool},
};
use examples_runner as _;
use rtic::app;

// Declare a pool of 128-byte memory blocks
pool!(P: [u8; 128]);

#[app(device = examples_runner::pac, dispatchers = [SSI0, QEI0])]
mod app {
    use crate::{Box, Pool};
    use examples_runner::{println, exit};
    use examples_runner::pac::Interrupt;

    // Import the memory pool into scope
    use super::P;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init(local = [memory: [u8; 512] = [0; 512]])]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        // Increase the capacity of the memory pool by ~4
        P::grow(cx.local.memory);

        rtic::pend(Interrupt::I2C0);

        (Shared {}, Local {}, init::Monotonics())
    }

    #[task(binds = I2C0, priority = 2)]
    fn i2c0(_: i2c0::Context) {
        // claim a memory block, initialize it and ..
        let x = P::alloc().unwrap().init([0u8; 128]);

        // .. send it to the `foo` task
        foo::spawn(x).ok().unwrap();

        // send another block to the task `bar`
        bar::spawn(P::alloc().unwrap().init([0u8; 128]))
            .ok()
            .unwrap();
    }

    #[task]
    fn foo(_: foo::Context, x: Box<P>) {
        println!("foo");

        // explicitly return the block to the pool
        drop(x);

        exit();
    }

    #[task(priority = 2)]
    fn bar(_: bar::Context, _x: Box<P>) {
        println!("bar");

        // this is done automatically so we can omit the call to `drop`
        // drop(x);
    }
}
