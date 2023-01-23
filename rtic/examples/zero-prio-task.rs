//! examples/zero-prio-task.rs

#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![deny(missing_docs)]

use core::marker::PhantomData;
use panic_semihosting as _;

/// Does not impl send
pub struct NotSend {
    _0: PhantomData<*const ()>,
}

#[rtic::app(device = lm3s6965, peripherals = true)]
mod app {
    use super::NotSend;
    use core::marker::PhantomData;
    use cortex_m_semihosting::{debug, hprintln};

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
        hprintln!("init");

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
        hprintln!("hello from async");
    }

    #[task(priority = 0, shared = [x])]
    async fn async_task2(_: async_task2::Context) {
        hprintln!("hello from async2");

        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }
}
