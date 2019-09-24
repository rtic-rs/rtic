//! examples/idle.rs
#![feature(generator_trait)]
#![feature(generators)]
#![feature(never_type)]
#![feature(type_alias_impl_trait)]
#![no_main]
#![no_std]

use core::ops::Generator;
use cortex_m_semihosting::{debug, hprintln};
use lm3s6965::Interrupt;
use panic_semihosting as _;
use rtfm::{Exclusive, Mutex};

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    struct Resources {
        #[init(0)]
        shared: u32,
    }

    #[init]
    fn init(_: init::Context) {
        hprintln!("init").unwrap();
        rtfm::pend(Interrupt::GPIOA);
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {

        hprintln!("idle").unwrap();
        rtfm::pend(Interrupt::GPIOA);
        rtfm::pend(Interrupt::GPIOA);

        debug::exit(debug::EXIT_SUCCESS);

        loop {}
    }

    #[task(binds = GPIOA, resources = [shared])]
    // fn foo1(ctx: &'static mut foo1::Context) -> impl Generator<Yield = (), Return = !> {
    fn foo1(ctx: foo1::Context) -> impl Generator<Yield = (), Return = !> {
        let mut local = 0;
        //let shared_res = Exclusive(ctx.resources.shared);
       // static shared_res : &mut u32 = ctx.resources.shared;
        move || loop {
            hprintln!("foo1_1 {}", local).unwrap();
            local += 1;
            // shared_res.lock(|s| hprintln!("s {}", s));
         //   *shared_res += 1;
            // *ctx.resources.shared += 1;
            yield;
            hprintln!("foo1_2 {}", local).unwrap();
            local += 1;
            yield;
        }
    }

    // #[task(binds = GPIOB, resources = [shared], priority = 2)]
    // fn foo2(ctx: foo2::Context) {}
};
