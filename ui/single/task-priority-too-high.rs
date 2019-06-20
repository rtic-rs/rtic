#![no_main]

use rtfm::app;

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    #[init]
    fn init(_: init::Context) {}

    #[task(binds = GPIOA, priority = 1)]
    fn gpioa(_: gpioa::Context) {}

    #[task(binds = GPIOB, priority = 2)]
    fn gpiob(_: gpiob::Context) {}

    #[task(binds = GPIOC, priority = 3)]
    fn gpioc(_: gpioc::Context) {}

    #[task(binds = GPIOD, priority = 4)]
    fn gpiod(_: gpiod::Context) {}

    #[task(binds = GPIOE, priority = 5)]
    fn gpioe(_: gpioe::Context) {}

    #[task(binds = UART0, priority = 6)]
    fn uart0(_: uart0::Context) {}

    #[task(binds = UART1, priority = 7)]
    fn uart1(_: uart1::Context) {}

    // OK, this is the maximum priority supported by the device
    #[task(binds = SSI0, priority = 8)]
    fn ssi0(_: ssi0::Context) {}

    // this value is too high!
    #[task(binds = I2C0, priority = 9)]
    fn i2c0(_: i2c0::Context) {}
};
