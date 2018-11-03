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

    #[interrupt]
    fn UART0() -> u32 {
        //~^ ERROR `interrupt` handlers must have type signature `[unsafe] fn()`
        0
    }
};
