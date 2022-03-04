#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use examples_runner as _;

#[rtic::app(device = examples_runner::pac)]
mod app {
    use examples_runner::exit;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        rtic::pend(examples_runner::pac::Interrupt::UART0);

        (Shared {}, Local {}, init::Monotonics())
    }

    #[task(binds = UART0)]
    fn taskmain(_: taskmain::Context) {
        exit();
    }
}
