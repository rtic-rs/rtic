#![no_main]
#![no_std]

extern crate lm3s6965;
extern crate panic_halt;
extern crate rtfm;

use rtfm::app;

#[app(device = lm3s6965)] //~ error evaluation of constant value failed
const APP: () = {
    #[init]
    fn init(_: init::Context) {}

    // OK, this is the maximum priority supported by the device
    #[interrupt(priority = 8)]
    fn UART0(_: UART0::Context) {}

    // this value is too high!
    #[interrupt(priority = 9)]
    fn UART1(_: UART1::Context) {}
};
