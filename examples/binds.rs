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

    #[resources]
    struct Resources {
        // A local (move), late resource
        #[task_local]
        #[init(0)]
        times: u32,
    }

    #[init]
    fn init(_: init::Context) -> (init::LateResources, init::Monotonics) {
        rtic::pend(Interrupt::UART0);

        hprintln!("init").unwrap();

        (init::LateResources {}, init::Monotonics())
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        hprintln!("idle").unwrap();

        rtic::pend(Interrupt::UART0);

        debug::exit(debug::EXIT_SUCCESS);

        loop {
            cortex_m::asm::nop();
        }
    }

    #[task(binds = UART0, resources = [times])]
    fn foo(cx: foo::Context) {
        let times = cx.resources.times;
        *times += 1;

        hprintln!(
            "foo called {} time{}",
            *times,
            if *times > 1 { "s" } else { "" }
        )
        .unwrap();
    }
}
