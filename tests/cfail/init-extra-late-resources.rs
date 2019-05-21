#![no_main]
#![no_std]

extern crate lm3s6965;
extern crate panic_halt;
extern crate rtfm;

use rtfm::app;

#[app(device = lm3s6965)]
const APP: () = {
    #[init]
    fn init(_: init::Context) -> init::LateResources {}
    //~^ error: `init` signature must be `fn(init::Context)` if there are no late resources
};
