#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_main]
#![no_std]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
extern crate cortex_m_rtfm as rtfm;
extern crate panic_abort;
extern crate stm32f103xx;

use cortex_m::asm;
use rt::ExceptionFrame;
use rtfm::app;

app! {
    device: stm32f103xx,

    tasks: {
        exti0: {
            interrupt: EXTI0,
        },
    },
}

#[inline(always)]
fn init(mut _ctxt: init::Context) -> init::LateResources {
    init::LateResources {}
}

#[inline(always)]
fn idle(_ctxt: idle::Context) -> ! {
    loop {
        asm::wfi();
    }
}

fn exti0(_ctxt: exti0::Context) {}

exception!(HardFault, hard_fault);

#[inline(always)]
fn hard_fault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

exception!(*, default_handler);

#[inline(always)]
fn default_handler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
