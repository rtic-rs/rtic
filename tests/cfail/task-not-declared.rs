#![no_main]
#![no_std]

extern crate lm3s6965;
extern crate panic_halt;
extern crate rtfm;

use rtfm::app;

#[app(device = lm3s6965)]
const APP: () = {
    #[init(spawn = [X])] //~ ERROR this task has NOT been declared
    fn init() {}
};
