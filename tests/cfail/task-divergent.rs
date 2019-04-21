#![no_main]
#![no_std]

extern crate lm3s6965;
extern crate panic_halt;
extern crate rtfm;

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    #[init]
    fn init(_: init::Context) {}

    #[task]
    fn foo(_: foo::Context) -> ! {
        //~^ ERROR this `task` handler must have type signature `fn(foo::Context, ..)`
        loop {}
    }

    extern "C" {
        fn UART0();
    }
};
