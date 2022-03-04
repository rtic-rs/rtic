//! `examples/not-sync.rs`

// #![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use core::marker::PhantomData;
use examples_runner as _;

pub struct NotSync {
    _0: PhantomData<*const ()>,
}

unsafe impl Send for NotSync {}

#[rtic::app(device = examples_runner::pac, dispatchers = [SSI0])]
mod app {
    use super::NotSync;
    use core::marker::PhantomData;
    use examples_runner::exit;

    #[shared]
    struct Shared {
        shared: NotSync,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        (
            Shared {
                shared: NotSync { _0: PhantomData },
            },
            Local {},
            init::Monotonics(),
        )
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        exit();
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
