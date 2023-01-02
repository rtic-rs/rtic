//! `examples/not-sync.rs`

// #![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use core::marker::PhantomData;
use panic_semihosting as _;

pub struct NotSync {
    _0: PhantomData<*const ()>,
}

unsafe impl Send for NotSync {}

#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    use super::NotSync;
    use core::marker::PhantomData;
    use cortex_m_semihosting::debug;

    #[shared]
    struct Shared {
        shared: NotSync,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator

        (
            Shared {
                shared: NotSync { _0: PhantomData },
            },
            Local {},
            init::Monotonics(),
        )
    }

    #[task(shared = [&shared])]
    fn foo(c: foo::Context) {
        let _: &NotSync = c.shared.shared;
    }

    #[task(shared = [&shared])]
    fn bar(c: bar::Context) {
        let _: &NotSync = c.shared.shared;
    }
}
