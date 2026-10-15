//! examples/pool.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use heapless::{
    box_pool,
    pool::boxed::{Box, BoxBlock},
};
use panic_semihosting as _;
use rtic::app;

// Declare a pool of 128-byte memory blocks
box_pool!(P: [u8; 128]);

#[app(device = lm3s6965, dispatchers = [SSI0, QEI0])]
mod app {
    use crate::{Box, BoxBlock};
    use cortex_m_semihosting::debug;
    use lm3s6965::Interrupt;
    // Import the memory pool into scope
    use super::P;

    const BLOCK: BoxBlock<[u8; 128]> = BoxBlock::new();

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init(local = [memory: [BoxBlock<[u8; 128]>; 4] = [BLOCK; 4]])]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        // Increase the capacity of the memory pool by 4 blocks
        for block in cx.local.memory.iter_mut() {
            P.manage(block);
        }

        rtic::pend(Interrupt::I2C0);

        (Shared {}, Local {}, init::Monotonics())
    }

    #[task(binds = I2C0, priority = 2)]
    fn i2c0(_: i2c0::Context) {
        // claim a memory block, initialize it and ..
        let x = P.alloc([0u8; 128]).unwrap();

        // .. send it to the `foo` task
        foo::spawn(x).ok().unwrap();

        // send another block to the task `bar`
        bar::spawn(P.alloc([0u8; 128]).unwrap()).ok().unwrap();
    }

    #[task]
    fn foo(_: foo::Context, x: Box<P>) {
        // explicitly return the block to the pool
        drop(x);

        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }

    #[task(priority = 2)]
    fn bar(_: bar::Context, _x: Box<P>) {
        // this is done automatically so we can omit the call to `drop`
        // drop(x);
    }
}
