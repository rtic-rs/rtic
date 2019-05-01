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

    #[exception]
    fn SVCall(_: SVCall::Context, undef: u32) {
        //~^ ERROR this `exception` handler must have type signature `fn(SVCall::Context)`
    }
};
