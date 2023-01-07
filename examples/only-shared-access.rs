//! examples/only-shared-access.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [UART0, UART1])]
mod app {
    use cortex_m_semihosting::{debug, hprintln};

    #[shared]
    struct Shared {
        key: u32,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        foo::spawn().unwrap();
        bar::spawn().unwrap();

        (Shared { key: 0xdeadbeef }, Local {})
    }

    #[task(shared = [&key])]
    async fn foo(cx: foo::Context) {
        let key: &u32 = cx.shared.key;
        hprintln!("foo(key = {:#x})", key).unwrap();

        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
    }

    #[task(priority = 2, shared = [&key])]
    async fn bar(cx: bar::Context) {
        hprintln!("bar(key = {:#x})", cx.shared.key).unwrap();
    }
}
