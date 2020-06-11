#![no_main]

#[rtic::app(device = lm3s6965)]
const APP: () = {
    #[task(binds = UART0)]
    fn a(_: a::Context) {}

    extern "C" {
        fn UART0();
    }
};
