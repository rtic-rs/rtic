#![no_main]
#![no_std]

use panic_halt as _;

#[rtfm::app(cores = 2, device = heterogeneous)]
const APP: () = {
    #[init(core = 0, spawn = [foo])]
    fn init(c: init::Context) {
        c.spawn.foo().ok();
    }

    #[task(core = 1)]
    fn foo(_: foo::Context) {}

    extern "C" {
        #[core = 1]
        fn I0();
    }
};
