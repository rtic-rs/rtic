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

    // OK, this is the minimum priority that tasks can have
    #[interrupt(priority = 1)]
    fn UART0() {}

    // this value is too low!
    #[interrupt(priority = 0)] //~ error this literal must be in the range 1...255
    fn UART1() {}
};
