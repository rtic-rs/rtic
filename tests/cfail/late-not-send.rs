//! `init` has a static priority of `0`. Initializing resources from it is equivalent to sending a
//! message to the task that will own the resource

#![feature(extern_crate_item_prelude)] // ???
#![no_main]
#![no_std]

extern crate lm3s6965;
extern crate panic_halt;
extern crate rtfm;

use core::marker::PhantomData;

use rtfm::app;

struct NotSend {
    _0: PhantomData<*const ()>,
}

#[app(device = lm3s6965)] //~ ERROR `*const ()` cannot be sent between threads safely
const APP: () = {
    static mut X: NotSend = ();

    #[init]
    fn init() {
        X = NotSend { _0: PhantomData };
    }

    #[interrupt(resources = [X])]
    fn UART0() {}
};
