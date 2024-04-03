//! examples/pool.rs

#![no_main]
#![no_std]
#![deny(warnings)]

use heapless::{
    box_pool,
    pool::boxed::{Box, BoxBlock},
};
use panic_semihosting as _;
use rtic::app;

// Declare a pool containing 8-byte memory blocks
box_pool!(P: u8);

const POOL_CAPACITY: usize = 512;

#[app(device = lm3s6965, dispatchers = [SSI0, QEI0])]
mod app {
    use crate::{Box, BoxBlock, POOL_CAPACITY};
    use cortex_m_semihosting::debug;
    use lm3s6965::Interrupt;

    // Import the memory pool into scope
    use crate::P;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    const BLOCK: BoxBlock<u8> = BoxBlock::new();

    #[init(local = [memory: [BoxBlock<u8>; POOL_CAPACITY] = [BLOCK; POOL_CAPACITY]])]
    fn init(cx: init::Context) -> (Shared, Local) {
        for block in cx.local.memory {
            // Give the 'static memory to the pool
            P.manage(block);
        }

        rtic::pend(Interrupt::I2C0);

        (Shared {}, Local {})
    }

    #[task(binds = I2C0, priority = 2)]
    fn i2c0(_: i2c0::Context) {
        // Claim 128 u8 blocks
        let x = P.alloc(128).unwrap();

        // .. send it to the `foo` task
        foo::spawn(x).ok().unwrap();

        // send another 128 u8 blocks to the task `bar`
        bar::spawn(P.alloc(128).unwrap()).ok().unwrap();
    }

    #[task]
    async fn foo(_: foo::Context, _x: Box<P>) {
        // explicitly return the block to the pool
        drop(_x);

        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }

    #[task(priority = 2)]
    async fn bar(_: bar::Context, _x: Box<P>) {
        // this is done automatically so we can omit the call to `drop`
        // drop(_x);
    }
}
