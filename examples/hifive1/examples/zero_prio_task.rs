//! zero priority task
#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]

use core::marker::PhantomData;
use hifive1::hal::e310x;
use riscv_rt as _;

/// Does not impl send
pub struct NotSend {
    _0: PhantomData<*const ()>,
}

#[cfg_attr(feature = "riscv-mecall-backend", rtic::app(device = e310x))]
#[cfg_attr(feature = "riscv-clint-backend", rtic::app(device = e310x, backend = H0))]
mod app {
    use super::e310x;
    use super::NotSend;
    use core::marker::PhantomData;
    use semihosting::{println, process::exit};

    #[shared]
    struct Shared {
        x: NotSend,
    }

    #[local]
    struct Local {
        y: NotSend,
    }

    #[init]
    fn init(_cx: init::Context) -> (Shared, Local) {
        println!("init");

        async_task::spawn().unwrap();
        async_task2::spawn().unwrap();

        (
            Shared {
                x: NotSend { _0: PhantomData },
            },
            Local {
                y: NotSend { _0: PhantomData },
            },
        )
    }

    #[task(priority = 0, shared = [x], local = [y])]
    async fn async_task(_: async_task::Context) {
        println!("hello from async");
    }

    #[task(priority = 0, shared = [x])]
    async fn async_task2(_: async_task2::Context) {
        println!("hello from async2");

        exit(0); // Exit QEMU simulator
    }
}
