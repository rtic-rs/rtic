#![no_main]
#![no_std]

extern crate lm3s6965;
extern crate panic_halt;
extern crate rtfm;

use core::marker::PhantomData;

use rtfm::app;

pub struct NotSend {
    _0: PhantomData<*const ()>,
}

unsafe impl Sync for NotSend {}

#[app(device = lm3s6965)] //~ ERROR cannot be sent between threads safely
const APP: () = {
    #[init(spawn = [foo])]
    fn init(_: init::Context) {}

    #[task]
    fn foo(_: foo::Context, _x: NotSend) {}

    extern "C" {
        fn UART0();
    }
};
