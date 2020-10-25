//! examples/systic.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965)]
mod app {
    use cortex_m_semihosting::{debug, hprintln};

    #[resources]
    struct Resources {
        #[init(0)]
        #[task_local]
        counter: i32,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        let mut syst = cx.core.SYST;
        syst.set_reload(100000);
        syst.enable_interrupt();
        syst.enable_counter();

        hprintln!("init").unwrap();
        init::LateResources {}
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        hprintln!("idle").unwrap();
        loop {
            cortex_m::asm::nop();
        }
    }

    #[task(binds = SysTick, resources = [counter])]
    fn systic(cx: systic::Context) {
        let counter = cx.resources.counter;

        *counter += 1;
        hprintln!("SysTick #{}", counter).unwrap();
        if *counter == 10 {
            debug::exit(debug::EXIT_SUCCESS);
        }
    }
}
