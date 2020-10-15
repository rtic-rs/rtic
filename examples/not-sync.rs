//! `examples/not-sync.rs`

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use core::marker::PhantomData;
use panic_halt as _;

pub struct NotSync {
    _0: PhantomData<*const ()>,
}

#[rtic::app(device = lm3s6965)]
mod app {
    use super::NotSync;
    use core::marker::PhantomData;
    use cortex_m_semihosting::debug;

    #[resources]
    struct Resources {
        #[init(NotSync { _0: PhantomData })]
        shared: NotSync,
    }

    #[init]
    fn init(_: init::Context) -> init::LateResources {
        debug::exit(debug::EXIT_SUCCESS);

        init::LateResources {}
    }

    #[task(resources = [&shared])]
    fn foo(c: foo::Context) {
        let _: &NotSync = c.resources.shared;
    }

    #[task(resources = [&shared])]
    fn bar(c: bar::Context) {
        let _: &NotSync = c.resources.shared;
    }

    // RTIC requires that unused interrupts are declared in an extern block when
    // using software tasks; these free interrupts will be used to dispatch the
    // software tasks.
    extern "C" {
        fn SSI0();
    }
}
