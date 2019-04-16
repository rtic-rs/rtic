#![no_main]
#![no_std]

extern crate lm3s6965;
extern crate panic_halt;
extern crate rtfm;

use rtfm::app;

#[app(device = lm3s6965)] //~ error evaluation of constant value failed
const APP: () = {
    #[init]
    fn init() {}

    // OK, this is the maximum priority supported by the device
    #[interrupt(priority = 8)]
    fn UART0() {}

    // this value is too high!
    #[interrupt(priority = 9)]
    fn UART1() {}
};
