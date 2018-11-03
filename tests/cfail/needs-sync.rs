#![feature(extern_crate_item_prelude)] // ???
#![no_main]
#![no_std]

extern crate lm3s6965;
extern crate panic_halt;
extern crate rtfm;

use core::marker::PhantomData;

use rtfm::app;

pub struct NotSync {
    _0: PhantomData<*const ()>,
}

unsafe impl Send for NotSync {}

#[app(device = lm3s6965)] //~ ERROR cannot be shared between threads safely
const APP: () = {
    static X: NotSync = NotSync { _0: PhantomData };

    #[init(spawn = [foo])]
    fn init() {}

    #[task(priority = 1, resources = [X])]
    fn foo() {}

    #[task(priority = 2, resources = [X])]
    fn bar() {}

    extern "C" {
        fn UART0();
        fn UART1();
    }
};
