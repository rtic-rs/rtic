#![no_main]
#![no_std]

extern crate lm3s6965;
extern crate panic_halt;
extern crate rtfm;

use rtfm::app;

#[app(device = lm3s6965)]
const APP: () = {
    static mut UNINITIALIZED: bool = ();

    #[init]
    fn init() {
        let x = || {
            // this is OK
            return 0;
        };

        return; //~ ERROR `init` is *not* allowed to early return

        UNINITIALIZED = true;
    }

    #[interrupt(resources = [UNINITIALIZED])]
    fn UART0() {
        if resources.UNINITIALIZED {
            // UB
        }
    }
};
