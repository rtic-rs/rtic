//! `examples/not-sync.rs`

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use core::marker::PhantomData;

use cortex_m_semihosting::debug;
use panic_halt as _;

pub struct NotSync {
    _0: PhantomData<*const ()>,
}

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    struct Resources {
        #[init(NotSync { _0: PhantomData })]
        shared: NotSync,
    }

    #[init]
    fn init(_: init::Context) {
        debug::exit(debug::EXIT_SUCCESS);
    }

    #[task(resources = [&shared])]
    fn foo(c: foo::Context) {
        let _: &NotSync = c.resources.shared;
    }

    #[task(resources = [&shared])]
    fn bar(c: bar::Context) {
        let _: &NotSync = c.resources.shared;
    }

    extern "C" {
        fn UART0();
    }
};
