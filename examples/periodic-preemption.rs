// # Pointers (old)
//
// ~96~ 80 bytes .bss
//
// # -Os
//
// init
// a(bl=16000000, now=16000249)
// b(bl=24000000, now=24000248)
// a(bl=32000000, now=32000249)
// b(bl=48000000, now=48000282)
// a(bl=48000000, now=48001731)
// a(bl=64000000, now=64000249)
// b(bl=72000000, now=72000248)
// a(bl=80000000, now=80000249)
// b(bl=96000000, now=96000282)
// a(bl=96000000, now=96001731)
//
// # -O3
//
// init
// a(bl=16000000, now=16000228)
// b(bl=24000000, now=24000231)
// a(bl=32000000, now=32000228)
// b(bl=48000000, now=48000257)
// a(bl=48000000, now=48001705)
// a(bl=64000000, now=64000228)
// b(bl=72000000, now=72000231)
// a(bl=80000000, now=80000228)
// b(bl=96000000, now=96000257)
// a(bl=96000000, now=96001705)
//
// # Indices (new)
//
// 48 bytes .bss
//
// ## -O3
//
// init
// a(bl=16000000, now=16000193)
// b(bl=24000000, now=24000196)
// a(bl=32000000, now=32000193)
// b(bl=48000000, now=48000225)
// a(bl=48000000, now=48001440)
// a(bl=64000000, now=64000193)
// b(bl=72000000, now=72000196)
// a(bl=80000000, now=80000193)
// b(bl=96000000, now=96000225)
// a(bl=96000000, now=96001440)
//
// ## -Os
//
// init
// a(bl=16000000, now=16000253)
// b(bl=24000000, now=24000251)
// a(bl=32000000, now=32000253)
// b(bl=48000000, now=48000283)
// a(bl=48000000, now=48001681)
// a(bl=64000000, now=64000253)
// b(bl=72000000, now=72000251)
// a(bl=80000000, now=80000253)
// b(bl=96000000, now=96000283)
// a(bl=96000000, now=96001681)

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
            resources: [ITM],
        },

        b: {
            schedule_after: [b],
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

    ctxt.tasks.a.schedule_now(&mut ctxt.priority).ok();
    ctxt.tasks.b.schedule_now(&mut ctxt.priority).ok();

    init::LateResources { ITM: ctxt.core.ITM }
}

fn a(mut ctxt: a::Context) {
    let now = DWT::get_cycle_count();

    let st = ctxt.scheduled_time;
    ctxt.resources.ITM.claim_mut(&mut ctxt.priority, |itm, _| {
        iprintln!(&mut itm.stim[0], "a(st={}, now={})", st, now);
    });

    ctxt.tasks.a.schedule_after(&mut ctxt.priority, 2 * S).ok();
}

fn b(mut ctxt: b::Context) {
    let now = DWT::get_cycle_count();

    let st = ctxt.scheduled_time;
    let t = &mut ctxt.priority;
    iprintln!(
        &mut ctxt.resources.ITM.borrow_mut(t).stim[0],
        "b(st={}, now={})",
        st,
        now
    );

    ctxt.tasks.b.schedule_after(t, 3 * S).ok();
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
