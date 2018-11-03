#![feature(extern_crate_item_prelude)] // ???
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
    fn init() {}

    #[task]
    fn foo(_x: NotSend) {}

    extern "C" {
        fn UART0();
    }
};
