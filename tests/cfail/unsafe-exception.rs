#![no_main]
#![no_std]

extern crate lm3s6965;
extern crate panic_halt;
extern crate rtfm;

use rtfm::app;

#[app(device = lm3s6965)]
const APP: () = {
    #[init]
    fn init(_: init::Context) {}

    #[exception(binds = SVCall)]
    unsafe fn foo(_: foo::Context) {}
    //~^ ERROR this `exception` handler must have type signature `fn(foo::Context)`
};
