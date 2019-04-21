#![no_main]
#![no_std]

extern crate lm3s6965;
extern crate panic_halt;
extern crate rtfm;

use rtfm::app;

#[app(device = lm3s6965)]
const APP: () = {
    #[init]
    fn init(_: init::Context) {
        #[cfg(never)]
        static mut FOO: u32 = 0;

        FOO; //~ ERROR cannot find value `FOO` in this scope
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        #[cfg(never)]
        static mut FOO: u32 = 0;

        FOO; //~ ERROR cannot find value `FOO` in this scope

        loop {}
    }

    #[exception]
    fn SVCall(_: SVCall::Context) {
        #[cfg(never)]
        static mut FOO: u32 = 0;

        FOO; //~ ERROR cannot find value `FOO` in this scope
    }

    #[interrupt]
    fn UART0(_: UART0::Context) {
        #[cfg(never)]
        static mut FOO: u32 = 0;

        FOO; //~ ERROR cannot find value `FOO` in this scope
    }

    #[task]
    fn foo(_: foo::Context) {
        #[cfg(never)]
        static mut FOO: u32 = 0;

        FOO; //~ ERROR cannot find value `FOO` in this scope
    }

    extern "C" {
        fn UART1();
    }
};
