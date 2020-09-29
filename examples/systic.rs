//! examples/systic.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m_semihosting::{debug, hprintln};
// use lm3s6965::Interrupt;
use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
const APP: () = {
    #[init]
    fn init(cx: init::Context) {
        let mut syst = cx.core.SYST;
        syst.set_reload(100000);
        syst.enable_interrupt();
        syst.enable_counter();

        hprintln!("init").unwrap();
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        hprintln!("idle").unwrap();
        loop {
            cortex_m::asm::nop();
        }
    }

    #[task(binds = SysTick)]
    fn t(_: t::Context) {
        static mut COUNTER: i32 = 0;
        *COUNTER += 1;
        hprintln!("SysTick #{}", COUNTER).unwrap();
        if *COUNTER == 10 {
            debug::exit(debug::EXIT_SUCCESS);
        }
    }
};
