// # Pointers (old)
//
// ~52~ 48 bytes .bss
//
// # -Os
//
// init
// a(st=8000000, now=8000180, input=0)
// a(st=16000000, now=16000180, input=1)
// a(st=24000000, now=24000180, input=2)
//
// # -O3
//
// init
// a(st=8000000, now=8000168, input=0)
// a(st=16000000, now=16000168, input=1)
// a(st=24000000, now=24000168, input=2)
//
// # Indices (new)
//
// 32 bytes .bss
//
// ## -O3
//
// init
// a(st=8000000, now=8000164, input=0)
// a(st=16000000, now=16000164, input=1)
//
// ## -Os
//
// init
// a(st=8000000, now=8000179, input=0)
// a(st=16000000, now=16000179, input=1)

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
            input: u16,
            resources: [ITM],
        },
    },
}

const MS: u32 = 8_000;
const S: u32 = 1_000 * MS;

#[inline(always)]
fn init(mut ctxt: init::Context) -> init::LateResources {
    iprintln!(&mut ctxt.core.ITM.stim[0], "init");

    ctxt.tasks.a.schedule_now(&mut ctxt.priority, 0).ok();

    init::LateResources { ITM: ctxt.core.ITM }
}

fn a(mut ctxt: a::Context) {
    let now = DWT::get_cycle_count();
    let input = ctxt.input;

    let st = ctxt.scheduled_time;
    let itm = ctxt.resources.ITM;
    iprintln!(
        &mut itm.stim[0],
        "a(st={}, now={}, input={})",
        st,
        now,
        input
    );

    ctxt.tasks
        .a
        .schedule_after(&mut ctxt.priority, 1 * S, input + 1)
        .ok();
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
