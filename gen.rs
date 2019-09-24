#![feature(prelude_import)]
//! examples/idle.rs
#![feature(generator_trait)]
#![feature(generators)]
#![feature(never_type)]
#![feature(type_alias_impl_trait)]
#![no_main]
#![no_std]
#[prelude_import]
use core::prelude::v1::*;
#[macro_use]
extern crate core;
#[macro_use]
extern crate compiler_builtins;

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

#[allow(non_snake_case)]
fn init(_: init::Context) {



        // Safe access to local `static mut` variable




        //    #[task(binds = GPIOA, resources = [x])]
        // in user code the return type will be `impl Generator<..>`
        // x.lock(|_| {});
        ::cortex_m_semihosting::export::hstdout_str("init\n").unwrap();
    rtfm::pend(Interrupt::GPIOA);
}
#[allow(non_snake_case)]
fn idle(idle::Locals { X, .. }: idle::Locals, _: idle::Context) -> ! {
    use rtfm::Mutex as _;
    let _x: &'static mut u32 = X;
    ::cortex_m_semihosting::export::hstdout_str("idle\n").unwrap();
    debug::exit(debug::EXIT_SUCCESS);
    loop  { }
}
type Generatorfoo1= impl  : Generator<Yield = (), Return = !>;
static mut GENERATOR_FOO1: core::mem::MaybeUninit<Generatorfoo1> =
    core::mem::MaybeUninit::uninit();
#[allow(non_snake_case)]
fn foo1(ctx: foo1::Context) -> Generatorfoo1 {
    use rtfm::Mutex as _;
    move ||
        loop  {
            ::cortex_m_semihosting::export::hstdout_str("foo1_1\n").unwrap();
            yield;
            ::cortex_m_semihosting::export::hstdout_str("foo1_2\n").unwrap();
            yield;
        }
}
#[allow(non_snake_case)]
#[doc = "Initialization function"]
pub mod init {
    #[doc = r" Execution context"]
    pub struct Context {
        #[doc = r" Core (Cortex-M) peripherals"]
        pub core: rtfm::export::Peripherals,
    }
    impl Context<> {
        #[inline(always)]
        pub unsafe fn new(core: rtfm::export::Peripherals) -> Self {
            Context{core,}
        }
    }
}
#[allow(non_snake_case)]
#[doc(hidden)]
pub struct idleLocals {
    X: &'static mut u32,
}
impl idleLocals<> {
    #[inline(always)]
    unsafe fn new() -> Self {
        static mut X: u32 = 0;
        idleLocals{X: &mut X,}
    }
}
#[allow(non_snake_case)]
#[doc = "Idle loop"]
pub mod idle {
    #[doc(inline)]
    pub use super::idleLocals as Locals;
    #[doc = r" Execution context"]
    pub struct Context {
    }
    impl Context<> {
        #[inline(always)]
        pub unsafe fn new(priority: &rtfm::export::Priority) -> Self {
            Context{}
        }
    }
}
#[allow(non_snake_case)]
#[doc = "Hardware task"]
pub mod foo1 {
    #[doc = r" Execution context"]
    pub struct Context {
    }
    impl Context<> {
        #[inline(always)]
        pub unsafe fn new(priority: &rtfm::export::Priority) -> Self {
            Context{}
        }
    }
}
#[doc = r" Implementation details"]
const APP: () =
    {
        #[doc =
          r" Always include the device crate which contains the vector table"]
        use lm3s6965 as _;
        #[allow(non_snake_case)]
        #[no_mangle]
        unsafe fn GPIOA() {
            const PRIORITY: u8 = 1u8;
            rtfm::export::run(PRIORITY,
                              ||
                                  {
                                      ::cortex_m_semihosting::export::hstdout_str("here\n").unwrap();
                                      core::pin::Pin::new(GENERATOR_FOO1.as_mut_ptr()).resume();
                                  });
        }
        #[no_mangle]
        unsafe extern "C" fn main() -> ! {
            rtfm::export::interrupt::disable();
            let mut core: rtfm::export::Peripherals =
                core::mem::transmute(());
            let _ = [(); ((1 << lm3s6965::NVIC_PRIO_BITS) - 1u8 as usize)];
            core.NVIC.set_priority(lm3s6965::Interrupt::GPIOA,
                                   rtfm::export::logical2hw(1u8,
                                                            lm3s6965::NVIC_PRIO_BITS));
            rtfm::export::NVIC::unmask(lm3s6965::Interrupt::GPIOA);
            let late = init(init::Context::new(core.into()));
            const PRIORITY: u8 = 1u8;
            unsafe {
                GENERATOR_FOO1.as_mut_ptr().write(foo1(foo1::Context::new(&rtfm::export::Priority::new(PRIORITY))));
            };
            rtfm::export::interrupt::enable();
            idle(idle::Locals::new(),
                 idle::Context::new(&rtfm::export::Priority::new(0)))
        }
    };
