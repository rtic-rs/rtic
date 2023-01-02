//! examples/binds.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

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
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        rtic::pend(Interrupt::UART0);

        hprintln!("init").unwrap();

        (Shared {}, Local {}, init::Monotonics())
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        hprintln!("idle").unwrap();

        rtic::pend(Interrupt::UART0);

        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator

        loop {
            cortex_m::asm::nop();
        }
    }

    #[task(binds = UART0, local = [times: u32 = 0])]
    fn foo(cx: foo::Context) {
        *cx.local.times += 1;

        hprintln!(
            "foo called {} time{}",
            *cx.local.times,
            if *cx.local.times > 1 { "s" } else { "" }
        )
        .unwrap();
    }
}
