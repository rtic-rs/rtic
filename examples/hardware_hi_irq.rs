//! examples/hardware_hi_irq.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![allow(const_err)]
#![no_main]
#![no_std]

use panic_semihosting as _;

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
        // Pends the I2C1 interrupt but its handler won't run until *after*
        // `init` returns because interrupts are disabled
        rtic::pend(Interrupt::I2C1); // equivalent to NVIC::pend

        hprintln!("init").unwrap();

        (Shared {}, Local {}, init::Monotonics())
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        // interrupts are enabled again; the `UART0` handler runs at this point

        hprintln!("idle").unwrap();

        rtic::pend(Interrupt::I2C1);

        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator

        loop {
            cortex_m::asm::nop();
        }
    }

    // ETHERNET has an interrupt vector index > 31
    // This is a test to check codegen for such cases
    #[task(binds = I2C1, local = [times: u32 = 0])]
    fn t1(cx: t1::Context) {
        // Safe access to local `static mut` variable
        *cx.local.times += 1;

        hprintln!(
            "t1 called {} time{}",
            *cx.local.times,
            if *cx.local.times > 1 { "s" } else { "" }
        )
        .unwrap();
    }
}
