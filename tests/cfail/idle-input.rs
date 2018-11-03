#![no_main]
#![no_std]

extern crate lm3s6965;
extern crate panic_halt;
extern crate rtfm;

use rtfm::app;

#[app(device = lm3s6965)]
const APP: () = {
    #[init]
    fn init() {}

    #[idle]
    fn idle(undef: u32) {
        //~^ ERROR `idle` must have type signature `[unsafe] fn() -> !`
    }
};
