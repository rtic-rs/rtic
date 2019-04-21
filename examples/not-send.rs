//! `examples/not-send.rs`

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_halt;

use core::marker::PhantomData;

use cortex_m_semihosting::debug;
use rtfm::app;

pub struct NotSend {
    _0: PhantomData<*const ()>,
}

#[app(device = lm3s6965)]
const APP: () = {
    static mut SHARED: Option<NotSend> = None;

    #[init(spawn = [baz, quux])]
    fn init(c: init::Context) {
        c.spawn.baz().unwrap();
        c.spawn.quux().unwrap();
    }

    #[task(spawn = [bar])]
    fn foo(c: foo::Context) {
        // scenario 1: message passed to task that runs at the same priority
        c.spawn.bar(NotSend { _0: PhantomData }).ok();
    }

    #[task]
    fn bar(_: bar::Context, _x: NotSend) {
        // scenario 1
    }

    #[task(priority = 2, resources = [SHARED])]
    fn baz(mut c: baz::Context) {
        // scenario 2: resource shared between tasks that run at the same priority
        *c.resources.SHARED = Some(NotSend { _0: PhantomData });
    }

    #[task(priority = 2, resources = [SHARED])]
    fn quux(mut c: quux::Context) {
        // scenario 2
        let _not_send = c.resources.SHARED.take().unwrap();

        debug::exit(debug::EXIT_SUCCESS);
    }

    extern "C" {
        fn UART0();
        fn UART1();
    }
};
