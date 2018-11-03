#![no_main]
#![no_std]

extern crate lm3s6965;
extern crate panic_halt;
extern crate rtfm;

use rtfm::app;

#[app(device = lm3s6965)]
const APP: () = {
    static mut X: u32 = ();

    #[init(resources = [X])] //~ ERROR late resources can NOT be assigned to `init`
    fn init() {}
};
