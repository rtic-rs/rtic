// # Pointers (old)
//
// ~52~ 40 bytes .bss
//
// ## -Os
//
// init
// a(st=8000000, now=8000180)
// a(st=16000000, now=16000180)
//
// ## -O3
//
// a(st=8000000, now=8000168)
// a(st=16000000, now=16000168)
//
// # Indices (new)
//
// 32 bytes .bss
//
// ## -Os
//
// init
// a(st=8000000, now=8000176)
// a(st=16000000, now=16000176)
//
// ## -O3
//
// init
// a(st=0, now=68)
// a(st=8000000, now=8000165)
// a(st=16000000, now=16000165)

#![deny(unsafe_code)]
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

    init: {
        schedule_now: [a],
    },

    free_interrupts: [EXTI0],

    tasks: {
        a: {
            schedule_after: [a],
            resources: [ITM],
        },
    },
}

const MS: u32 = 8_000;
const S: u32 = 1_000 * MS;

#[inline(always)]
fn init(mut ctxt: init::Context) -> init::LateResources {
    iprintln!(&mut ctxt.core.ITM.stim[0], "init");

    ctxt.tasks.a.schedule_now(&mut ctxt.priority).ok();

    init::LateResources { ITM: ctxt.core.ITM }
}

fn a(mut ctxt: a::Context) {
    let now = DWT::get_cycle_count();

    let st = ctxt.scheduled_time;
    let itm = ctxt.resources.ITM;
    iprintln!(&mut itm.stim[0], "a(st={}, now={})", st, now);

    ctxt.tasks.a.schedule_after(&mut ctxt.priority, 1 * S).ok();
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
