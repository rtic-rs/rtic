//! `examples/not-sync.rs`

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_halt;

use core::marker::PhantomData;

use cortex_m_semihosting::debug;

pub struct NotSync {
    _0: PhantomData<*const ()>,
}

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    static SHARED: NotSync = NotSync { _0: PhantomData };

    #[init]
    fn init(_: init::Context) {
        debug::exit(debug::EXIT_SUCCESS);
    }

    #[task(resources = [SHARED])]
    fn foo(c: foo::Context) {
        let _: &NotSync = c.resources.SHARED;
    }

    #[task(resources = [SHARED])]
    fn bar(c: bar::Context) {
        let _: &NotSync = c.resources.SHARED;
    }

    extern "C" {
        fn UART0();
    }
};
