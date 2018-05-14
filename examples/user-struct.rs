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

pub struct Foo(u32);

app! {
    device: stm32f103xx,

    resources: {
        static FOO: Foo = Foo(0);
        static BAR: Foo;
    },

    free_interrupts: [EXTI0],

    init: {
        schedule_now: [a],
        schedule_after: [b],
    },

    tasks: {
        a: {
            input: Foo,
        },

        b: {
            input: Foo,
        },
    },
}

#[inline(always)]
fn init(_ctxt: init::Context) -> init::LateResources {
    init::LateResources { BAR: Foo(0) }
}

#[inline(always)]
fn idle(_ctxt: idle::Context) -> ! {
    loop {
        asm::wfi();
    }
}

fn a(_ctxt: a::Context) {}

fn b(_ctxt: b::Context) {}

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
