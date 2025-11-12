//! examples/bouncy_trampoline.rs

#![no_std]
#![no_main]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]

use panic_semihosting as _;

// `examples/bouncy_trampoline.rs` testing trampoline feature 
#[rtic::app(device = lm3s6965)]
mod app {
    use cortex_m_semihosting::{debug, hprintln};
    use lm3s6965::Interrupt;

    #[shared]
    struct Shared {
        h_priority: u8,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        // Trigger SW0 interrupt to test NMI preemption
        rtic::pend(Interrupt::GPIOA);
        (Shared { h_priority: 0 }, Local {})
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        hprintln!("idle");

        // pend another interrupt to test trampoline
        cortex_m::peripheral::SCB::set_pendst();   
        hprintln!("idle end");
        
        debug::exit(debug::EXIT_SUCCESS);
        loop {
            cortex_m::asm::wfi();
        }
    }

    #[task(binds = SysTick, trampoline = GPIOC, priority = 2, shared = [h_priority])]
    fn sys_tick(_: sys_tick::Context) {
        hprintln!("SysTick start");
    }

    #[task(binds = GPIOA, priority = 1, shared = [h_priority])]
    fn gpioa(mut ctx: gpioa::Context) {
        ctx.shared.h_priority.lock(|_| {
            hprintln!("gpioa lock");
            cortex_m::peripheral::SCB::set_pendst();   
            cortex_m::asm::delay(100_000_000);
            hprintln!("gpioa unlock");
        });
    }

    #[task(binds = GPIOB, priority = 3, shared = [h_priority])]
    fn high_priority_task(_: high_priority_task::Context) {
        hprintln!("High priority task");
    }
}
