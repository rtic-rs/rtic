#![deny(warnings)]
#![feature(proc_macro)]
#![feature(proc_macro_gen)]
#![no_main]
#![no_std]

#[macro_use]
extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
extern crate cortex_m_rtfm as rtfm;
extern crate panic_abort;
extern crate stm32f103xx;

use cortex_m::peripheral::{DWT, ITM};
use rt::ExceptionFrame;
use rtfm::app;

app! {
    device: stm32f103xx,

    resources: {
        static ITM: ITM;
    },

    free_interrupts: [EXTI1],

    tasks: {
        exti0: {
            interrupt: EXTI0,
            schedule_after: [a],
            resources: [ITM],
        },

        a: {
            resources: [ITM],
        },
    },
}

const S: u32 = 8_000_000;

#[inline(always)]
fn init(ctxt: init::Context) -> init::LateResources {
    rtfm::_impl::trigger(stm32f103xx::Interrupt::EXTI0);

    init::LateResources { ITM: ctxt.core.ITM }
}

fn exti0(mut ctxt: exti0::Context) {
    let now = DWT::get_cycle_count();
    iprintln!(
        &mut ctxt.resources.ITM.stim[0],
        "exti0(st={}, now={})",
        ctxt.start_time,
        now
    );

    let t = &mut ctxt.priority;
    ctxt.tasks.a.schedule_after(t, 1 * S).ok();
}

fn a(ctxt: a::Context) {
    let now = DWT::get_cycle_count();
    iprintln!(
        &mut ctxt.resources.ITM.stim[0],
        "a(st={}, now={})",
        ctxt.scheduled_time,
        now
    );
}

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
