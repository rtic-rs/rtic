//! Working with resources in a generic fashion
#![deny(unsafe_code)]
#![deny(warnings)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m_rtfm as rtfm;
extern crate stm32f103xx;

use rtfm::{app, Resource, Threshold};
use stm32f103xx::{SPI1, GPIOA};

app! {
    device: stm32f103xx,

    resources: {
        static GPIOA: GPIOA;
        static SPI1: SPI1;
    },

    tasks: {
        EXTI0: {
            path: exti0,
            priority: 1,
            resources: [GPIOA, SPI1],
        },

        EXTI1: {
            path: exti1,
            priority: 2,
            resources: [GPIOA, SPI1],
        },
    },
}

fn init(p: init::Peripherals) -> init::LateResources {
    init::LateResources {
        GPIOA: p.device.GPIOA,
        SPI1: p.device.SPI1,
    }
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}

// A generic function that uses some resources
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

// This task needs critical sections to access the resources
fn exti0(t: &mut Threshold, r: EXTI0::Resources) {
    work(t, &r.GPIOA, &r.SPI1);
}

// This task has direct access to the resources
fn exti1(t: &mut Threshold, r: EXTI1::Resources) {
    work(t, &r.GPIOA, &r.SPI1);
}
