//! examples/executor-size.rs

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0, UART0], peripherals = true)]
mod app {
    use cortex_m_semihosting::{debug, hprintln};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        hprintln!("init, total executor size = {}", cx.executors_size);

        foo::spawn().ok();
        bar::spawn().ok();
        baz::spawn().ok();

        (Shared {}, Local {})
    }

    #[task]
    async fn foo(_cx: foo::Context) {}

    #[task]
    async fn bar(_cx: bar::Context) {}

    #[task]
    async fn baz(_cx: baz::Context) {
        debug::exit(debug::EXIT_SUCCESS);
    }
}
