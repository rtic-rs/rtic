//! examples/async-delay.rs

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0, UART0], peripherals = true)]
mod app {
    use cortex_m_semihosting::{debug, hprintln};
    use rtic_monotonics::systick::prelude::*;

    systick_monotonic!(Mono, 100);

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        hprintln!("init");

        Mono::start(cx.core.SYST, 12_000_000);

        foo::spawn().ok();
        bar::spawn().ok();
        baz::spawn().ok();

        (Shared {}, Local {})
    }

    #[task]
    async fn foo(_cx: foo::Context) {
        hprintln!("hello from foo");
        Mono::delay(100.millis()).await;
        hprintln!("bye from foo");
    }

    #[task]
    async fn bar(_cx: bar::Context) {
        hprintln!("hello from bar");
        Mono::delay(200.millis()).await;
        hprintln!("bye from bar");
    }

    #[task]
    async fn baz(_cx: baz::Context) {
        hprintln!("hello from baz");
        Mono::delay(300.millis()).await;
        hprintln!("bye from baz");

        debug::exit(debug::EXIT_SUCCESS);
    }
}
