#![no_main]
#![no_std]

extern crate lm3s6965;
extern crate panic_halt;
extern crate rtfm;

use rtfm::app;

#[app(device = lm3s6965)]
const APP: () = {
    static mut X: u32 = (); //~ ERROR late resources MUST be initialized at the end of `init`

    #[init]
    fn init() {}
};
