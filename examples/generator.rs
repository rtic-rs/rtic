//! examples/idle.rs
#![feature(generator_trait)]
#![feature(generators)]
#![feature(never_type)]
#![feature(type_alias_impl_trait)]
#![no_main]
#![no_std]

use core::ops::Generator;
// use core::mem::MaybeUninit;

// // #[task(binds = EXTI0, resources = [x])]
// // fn bar(cx: bar::Context) {
// //     let x = cx.resources.x;
// // }

// // expansion
// type Foo = impl Generator<Yield = (), Return = !>;

// static mut X: MaybeUninit<Foo> = MaybeUninit::uninit();

// fn main() {
//     // init();
//     unsafe {
//         X.as_mut_ptr().write(foo());
//     }
//     // idle();
// }

// #![deny(unsafe_code)]

use cortex_m_semihosting::{debug, hprintln};
use lm3s6965::Interrupt;
use panic_semihosting as _;

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    #[init]
    fn init(_: init::Context) {
        hprintln!("init").unwrap();
        rtfm::pend(Interrupt::GPIOA);
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        static mut X: u32 = 0;

        // Safe access to local `static mut` variable
        let _x: &'static mut u32 = X;

        hprintln!("idle").unwrap();

        debug::exit(debug::EXIT_SUCCESS);

        loop {}
    }

    //    #[task(binds = GPIOA, resources = [x])]
    // in user code the return type will be `impl Generator<..>`
    #[task(binds = GPIOA)]
    fn foo1(ctx: foo1::Context) -> impl Generator<Yield = (), Return = !> {
        move || loop {
            // x.lock(|_| {});
            hprintln!("foo1_1").unwrap();
            yield;
            hprintln!("foo1_2").unwrap();
            yield;
        }
    }
};
