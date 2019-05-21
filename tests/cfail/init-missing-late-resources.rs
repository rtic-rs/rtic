#![no_main]
#![no_std]

extern crate lm3s6965;
extern crate panic_halt;
extern crate rtfm;

use rtfm::app;

#[app(device = lm3s6965)]
const APP: () = {
    static mut X: i32 = ();

    #[init]
    fn init(_: init::Context) {}
    //~^ error: late resources have been specified so `init` must return `init::LateResources`
};
