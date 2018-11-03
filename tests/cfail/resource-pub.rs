#![no_main]
#![no_std]

extern crate lm3s6965;
extern crate panic_halt;
extern crate rtfm;

use rtfm::app;

#[app(device = lm3s6965)]
const APP: () = {
    pub static mut X: u32 = 0;
    //~^ ERROR resources must have inherited / private visibility

    #[init]
    fn init() {}
};
