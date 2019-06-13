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

    #[exception]
    fn SVCall(_: SVCall::Context) {
        #[cfg(never)]
        static mut FOO: u32 = 0;

        FOO;
    }

    #[interrupt]
    fn UART0(_: UART0::Context) {
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
