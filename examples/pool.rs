//! examples/pool.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use heapless::{
    pool,
    pool::singleton::{Box, Pool},
};
use panic_semihosting as _;
use rtic::app;

// Declare a pool of 128-byte memory blocks
pool!(P: [u8; 128]);

#[app(device = lm3s6965)]
mod app {
    use crate::{Box, Pool};
    use cortex_m_semihosting::{debug, hprintln};
    use lm3s6965::Interrupt;

    // Import the memory pool into scope
    use super::P;

    #[init]
    fn init(_: init::Context) -> init::LateResources {
        static mut MEMORY: [u8; 512] = [0; 512];

        // Increase the capacity of the memory pool by ~4
        P::grow(MEMORY);

        rtic::pend(Interrupt::I2C0);

        init::LateResources {}
    }

    #[task(binds = I2C0, priority = 2)]
    fn i2c0(_: i2c0::Context) {
        // claim a memory block, leave it uninitialized and ..
        let x = P::alloc().unwrap().freeze();

        // .. send it to the `foo` task
        foo::spawn(x).ok().unwrap();

        // send another block to the task `bar`
        bar::spawn(P::alloc().unwrap().freeze()).ok().unwrap();
    }

    #[task]
    fn foo(_: foo::Context, x: Box<P>) {
        hprintln!("foo({:?})", x.as_ptr()).unwrap();

        // explicitly return the block to the pool
        drop(x);

        debug::exit(debug::EXIT_SUCCESS);
    }

    #[task(priority = 2)]
    fn bar(_: bar::Context, x: Box<P>) {
        hprintln!("bar({:?})", x.as_ptr()).unwrap();

        // this is done automatically so we can omit the call to `drop`
        // drop(x);
    }

    // RTIC requires that unused interrupts are declared in an extern block when
    // using software tasks; these free interrupts will be used to dispatch the
    // software tasks.
    extern "C" {
        fn SSI0();
        fn QEI0();
    }
}
