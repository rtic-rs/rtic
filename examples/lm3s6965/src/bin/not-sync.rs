//! `examples/not-sync.rs`

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(missing_docs)]
#![feature(type_alias_impl_trait)]

use core::marker::PhantomData;
use panic_semihosting as _;

/// Not sync
pub struct NotSync {
    _0: PhantomData<*const ()>,
    data: u32,
}

unsafe impl Send for NotSync {}

#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    use super::NotSync;
    use core::marker::PhantomData;
    use cortex_m_semihosting::{debug, hprintln};

    #[shared]
    struct Shared {
        shared: NotSync,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        hprintln!("init");

        foo::spawn().unwrap();
        bar::spawn().unwrap();
        (
            Shared {
                shared: NotSync {
                    _0: PhantomData,
                    data: 13,
                },
            },
            Local {},
        )
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
        loop {}
    }

    #[task(shared = [&shared], priority = 1)]
    async fn foo(c: foo::Context) {
        let shared: &NotSync = c.shared.shared;
        hprintln!("foo a {}", shared.data);
    }

    #[task(shared = [&shared], priority = 1)]
    async fn bar(c: bar::Context) {
        let shared: &NotSync = c.shared.shared;
        hprintln!("bar a {}", shared.data);
    }
}
