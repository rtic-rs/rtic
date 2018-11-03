//! `examples/not-sync.rs`

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_halt;

use core::marker::PhantomData;

use cortex_m_semihosting::debug;
use rtfm::app;

pub struct NotSync {
    _0: PhantomData<*const ()>,
}

#[app(device = lm3s6965)]
const APP: () = {
    static SHARED: NotSync = NotSync { _0: PhantomData };

    #[init]
    fn init() {
        debug::exit(debug::EXIT_SUCCESS);
    }

    #[task(resources = [SHARED])]
    fn foo() {
        let _: &NotSync = resources.SHARED;
    }

    #[task(resources = [SHARED])]
    fn bar() {
        let _: &NotSync = resources.SHARED;
    }

    extern "C" {
        fn UART0();
    }
};
