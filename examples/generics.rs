//! Working with resources in a generic fashion

#![deny(unsafe_code)]
#![feature(proc_macro)]
#![no_std]

#[macro_use(task)]
extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::{app, Resource, Threshold};
use stm32f103xx::{SPI1, GPIOA};

app! {
    device: stm32f103xx,

    tasks: {
        EXTI0: {
            enabled: true,
            priority: 1,
            resources: [GPIOA, SPI1],
        },

        EXTI1: {
            enabled: true,
            priority: 2,
            resources: [GPIOA, SPI1],
        },
    },
}

fn init(_p: init::Peripherals) {}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

// a generic function to use resources in any task (regardless of its priority)
fn work<G, S>(t: &mut Threshold, gpioa: &G, spi1: &S)
where
    G: Resource<Data = GPIOA>,
    S: Resource<Data = SPI1>,
{
    gpioa.claim(t, |_gpioa, t| {
        // drive NSS low

        spi1.claim(t, |_spi1, _| {
            // transfer data
        });

        // drive NSS high
    });
}

task!(EXTI0, exti0);

// this task needs critical sections to access the resources
fn exti0(t: &mut Threshold, r: EXTI0::Resources) {
    work(t, &r.GPIOA, &r.SPI1);
}

task!(EXTI1, exti1);

// this task has direct access to the resources
fn exti1(t: &mut Threshold, r: EXTI1::Resources) {
    work(t, r.GPIOA, r.SPI1);
}
