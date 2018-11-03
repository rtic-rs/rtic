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

    #[exception]
    fn NonMaskableInt() {
        //~^ ERROR only exceptions with configurable priority can be used as hardware tasks
    }
};
