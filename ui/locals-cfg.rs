#![no_main]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    #[init]
    fn init(_: init::Context) -> (init::LateResources, init::Monotonics) {
        #[cfg(never)]
        static mut FOO: u32 = 0;

        FOO;

        (init::LateResources {}, init::Monotonics())
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        #[cfg(never)]
        static mut FOO: u32 = 0;

        FOO;

        loop {}
    }

    #[task(binds = SVCall)]
    fn svcall(_: svcall::Context) {
        #[cfg(never)]
        static mut FOO: u32 = 0;

        FOO;
    }

    #[task(binds = UART0)]
    fn uart0(_: uart0::Context) {
        #[cfg(never)]
        static mut FOO: u32 = 0;

        FOO;
    }

    #[task]
    fn foo(_: foo::Context) {
        #[cfg(never)]
        static mut FOO: u32 = 0;

        FOO;
    }
}
