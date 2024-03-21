//! examples/binds.rs

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]

use panic_semihosting as _;

// `examples/interrupt.rs` rewritten to use `binds`
#[rtic::app(device = lm3s6965)]
mod app {
    use cortex_m_semihosting::{debug, hprintln};
    use lm3s6965::Interrupt;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        rtic::pend(Interrupt::UART0);

        hprintln!("init");

        (Shared {}, Local {})
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        hprintln!("idle");

        rtic::pend(Interrupt::UART0);

        loop {
            cortex_m::asm::nop();
            debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
        }
    }

    #[task(binds = UART0, local = [times: u32 = 0])]
    fn foo(cx: foo::Context) {
        *cx.local.times += 1;

        hprintln!(
            "foo called {} time{}",
            *cx.local.times,
            if *cx.local.times > 1 { "s" } else { "" }
        );
    }
}
