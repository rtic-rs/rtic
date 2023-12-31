//! examples/async-delay.rs

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]
#![feature(type_alias_impl_trait)]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0, UART0], peripherals = true)]
mod app {
    use cortex_m_semihosting::{debug, hprintln};
    use rtic_monotonics::systick::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        hprintln!("init");

        let systick_token = rtic_monotonics::create_systick_token!();
        Systick::start(cx.core.SYST, 12_000_000, systick_token);

        foo::spawn().ok();
        bar::spawn().ok();
        baz::spawn().ok();

        (Shared {}, Local {})
    }

    #[task]
    async fn foo(_cx: foo::Context) {
        hprintln!("hello from foo");
        Systick::delay(100.millis()).await;
        hprintln!("bye from foo");
    }

    #[task]
    async fn bar(_cx: bar::Context) {
        hprintln!("hello from bar");
        Systick::delay(200.millis()).await;
        hprintln!("bye from bar");
    }

    #[task]
    async fn baz(_cx: baz::Context) {
        hprintln!("hello from baz");
        Systick::delay(300.millis()).await;
        hprintln!("bye from baz");

        debug::exit(debug::EXIT_SUCCESS);
    }
}
