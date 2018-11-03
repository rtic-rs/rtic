//! This is equivalent to the `late-not-send` cfail test

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

#[app(device = lm3s6965)] //~ ERROR `*const ()` cannot be sent between threads safely
const APP: () = {
    static mut X: Option<NotSend> = None;

    #[init(resources = [X])]
    fn init() {
        *resources.X = Some(NotSend { _0: PhantomData })
    }

    #[interrupt(resources = [X])]
    fn UART0() {}
};
