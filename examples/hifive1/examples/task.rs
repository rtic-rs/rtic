//! zero priority task
#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]

use hifive1::hal::e310x;
use riscv_rt as _;

#[cfg_attr(feature = "riscv-mecall-backend", rtic::app(device = e310x))]
#[cfg_attr(feature = "riscv-clint-backend", rtic::app(device = e310x, backend = H0))]
mod app {
    use super::e310x;
    use semihosting::{println, process::exit};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        foo::spawn().unwrap();

        (Shared {}, Local {})
    }

    #[task]
    async fn foo(_: foo::Context) {
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
    async fn bar(_: bar::Context) {
        println!("bar");

        exit(0); // Exit QEMU simulator
    }

    #[task(priority = 2)]
    async fn baz(_: baz::Context) {
        println!("baz");
    }
}
