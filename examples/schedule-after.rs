#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![feature(proc_macro_gen)]
#![no_main]
#![no_std]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
extern crate cortex_m_rtfm as rtfm;
extern crate panic_abort;
extern crate stm32f103xx;

use rt::ExceptionFrame;
use rtfm::app;

app! {
    device: stm32f103xx,

    free_interrupts: [EXTI1],

    tasks: {
        exti0: {
            interrupt: EXTI0,
            schedule_after: [a],
        },

        a: {},
    },
}

const S: u32 = 8_000_000;

#[inline(always)]
fn init(_ctxt: init::Context) -> init::LateResources {
    init::LateResources {}
}

fn exti0(mut ctxt: exti0::Context) {
    ctxt.tasks.a.schedule_after(&mut ctxt.priority, 1 * S).ok();
}

fn a(_ctxt: a::Context) {}

exception!(HardFault, hard_fault);

fn hard_fault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

exception!(*, default_handler);

fn default_handler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
