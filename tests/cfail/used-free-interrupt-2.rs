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

    #[interrupt(binds = UART0)] //~ ERROR free interrupts (`extern { .. }`) can't be used as interrupt handlers
    fn foo(_: foo::Context) {}

    extern "C" {
        fn UART0();
    }
};
