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

    #[task(
        priority = 1,
        priority = 2, //~ ERROR argument appears more than once
    )]
    fn foo(_: foo::Context) {}

    extern "C" {
        fn UART0();
    }
};
