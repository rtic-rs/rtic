#![no_main]

#[rtic::app(device = lm3s6965, dispatchers = [UART0])]
mod app {
    #[task(binds = UART0)]
    fn a(_: a::Context) {}
}
