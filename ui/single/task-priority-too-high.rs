#![no_main]

use rtfm::app;

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    #[init]
    fn init(_: init::Context) {}

    #[interrupt(priority = 1)]
    fn GPIOA(_: GPIOA::Context) {}

    #[interrupt(priority = 2)]
    fn GPIOB(_: GPIOB::Context) {}

    #[interrupt(priority = 3)]
    fn GPIOC(_: GPIOC::Context) {}

    #[interrupt(priority = 4)]
    fn GPIOD(_: GPIOD::Context) {}

    #[interrupt(priority = 5)]
    fn GPIOE(_: GPIOE::Context) {}

    #[interrupt(priority = 6)]
    fn UART0(_: UART0::Context) {}

    #[interrupt(priority = 7)]
    fn UART1(_: UART1::Context) {}

    // OK, this is the maximum priority supported by the device
    #[interrupt(priority = 8)]
    fn SSI0(_: SSI0::Context) {}

    // this value is too high!
    #[interrupt(priority = 9)]
    fn I2C0(_: I2C0::Context) {}
};
