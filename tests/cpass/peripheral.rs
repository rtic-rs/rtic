//! Core and device peripherals
#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate lm3s6965;
extern crate panic_halt;
extern crate rtfm;

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    #[init]
    fn init(c: init::Context) {
        let _: rtfm::Peripherals = c.core;
        let _: lm3s6965::Peripherals = c.device;
    }
};
