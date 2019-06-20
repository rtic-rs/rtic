#![no_main]

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    #[init]
    fn init(_: init::Context) {
        #[cfg(never)]
        static mut FOO: u32 = 0;

        FOO;
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

    extern "C" {
        fn UART1();
    }
};
