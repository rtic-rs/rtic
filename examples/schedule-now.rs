// # Pointers (old)
//
// ~40~ 32 bytes .bss
//
// # Indices (new)
//
// 12 bytes .bss

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
extern crate panic_semihosting;
extern crate stm32f103xx;

use cortex_m::asm;
use rt::ExceptionFrame;
use rtfm::app;

app! {
    device: stm32f103xx,

    init: {
        schedule_now: [a],
    },

    free_interrupts: [EXTI1],

    tasks: {
        exti0: {
            interrupt: EXTI0,
            schedule_now: [a],
        },

        a: {},
    },
}

#[inline(always)]
fn init(mut ctxt: init::Context) -> init::LateResources {
    ctxt.tasks.a.schedule_now(&mut ctxt.priority).unwrap();

    init::LateResources {}
}

fn exti0(mut ctxt: exti0::Context) {
    ctxt.tasks.a.schedule_now(&mut ctxt.priority).unwrap();
}

fn a(_ctxt: a::Context) {
    asm::bkpt();
}

exception!(HardFault, hard_fault);

fn hard_fault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

exception!(*, default_handler);

fn default_handler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
