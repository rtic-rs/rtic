// # Pointers (old)
//
// ~104~ 88 bytes .bss
//
// ## -Os
//
// a(st=16000000, now=16000248, input=0)
// b(st=24000000, now=24000251, input=0)
// a(st=32000000, now=32000248, input=1)
// b(st=48000000, now=48000283, input=1)
// a(st=48000000, now=48002427, input=2)
// a(st=64000000, now=64000248, input=3)
// b(st=72000000, now=72000251, input=2)
// a(st=80000000, now=80000248, input=4)
// b(st=96000000, now=96000283, input=3)
// a(st=96000000, now=96002427, input=5)
//
// ## -O3
//
// init
// a(st=16000000, now=16000231, input=0)
// b(st=24000000, now=24000230, input=0)
// a(st=32000000, now=32000231, input=1)
// b(st=48000000, now=48000259, input=1)
// a(st=48000000, now=48002397, input=2)
// a(st=64000000, now=64000231, input=3)
// b(st=72000000, now=72000230, input=2)
// a(st=80000000, now=80000231, input=4)
// b(st=96000000, now=96000259, input=3)
// a(st=96000000, now=96002397, input=5)
//
// # Indices (new)
//
// 56 bytes .bss
//
// ## -O3
//
// init
// a(st=16000000, now=16000193, input=0)
// b(st=24000000, now=24000196, input=0)
// a(st=32000000, now=32000193, input=1)
// b(st=48000000, now=48000225, input=1)
// a(st=48000000, now=48001958, input=2)
// a(st=64000000, now=64000193, input=3)
// b(st=72000000, now=72000196, input=2)
// a(st=80000000, now=80000193, input=4)
// b(st=96000000, now=96000225, input=3)
// a(st=96000000, now=96001958, input=5)
//
// ## -Os
//
// init
// a(st=16000000, now=16000257, input=0)
// b(st=24000000, now=24000252, input=0)
// a(st=32000000, now=32000257, input=1)
// b(st=48000000, now=48000284, input=1)
// a(st=48000000, now=48002326, input=2)
// a(st=64000000, now=64000257, input=3)
// b(st=72000000, now=72000252, input=2)
// a(st=80000000, now=80000257, input=4)
// b(st=96000000, now=96000284, input=3)
// a(st=96000000, now=96002326, input=5)

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
        schedule_now: [a, b],
    },

    free_interrupts: [EXTI0, EXTI1],

    tasks: {
        a: {
            schedule_after: [a],
            input: u16,
            resources: [ITM],
        },

        b: {
            schedule_after: [b],
            input: u16,
            priority: 2,
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
    ctxt.tasks.b.schedule_now(&mut ctxt.priority, 0).ok();

    init::LateResources { ITM: ctxt.core.ITM }
}

fn a(mut ctxt: a::Context) {
    let now = DWT::get_cycle_count();

    let input = ctxt.input;
    let st = ctxt.scheduled_time;
    ctxt.resources.ITM.claim_mut(&mut ctxt.priority, |itm, _| {
        iprintln!(
            &mut itm.stim[0],
            "a(st={}, now={}, input={})",
            st,
            now,
            input
        );
    });

    ctxt.tasks
        .a
        .schedule_after(&mut ctxt.priority, 2 * S, input + 1)
        .ok();
}

fn b(mut ctxt: b::Context) {
    let now = DWT::get_cycle_count();

    let st = ctxt.scheduled_time;
    let input = ctxt.input;
    let t = &mut ctxt.priority;
    iprintln!(
        &mut ctxt.resources.ITM.borrow_mut(t).stim[0],
        "b(st={}, now={}, input={})",
        st,
        now,
        input,
    );

    ctxt.tasks.b.schedule_after(t, 3 * S, input + 1).ok();
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
