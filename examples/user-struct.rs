#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
extern crate panic_abort;
extern crate stm32f103xx;

use cortex_m::asm;
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
        async: [a],
        async_after: [b],
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
