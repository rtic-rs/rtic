#![feature(prelude_import)]
//! examples/idle.rs
#![feature(generator_trait)]
#![feature(generators)]
#![feature(never_type)]
#![feature(type_alias_impl_trait)]
#![feature(const_fn)]
#![feature(fmt_internals)]
#![feature(compiler_builtins_lib)]
#![no_main]
#![no_std]
#[rustfmt::skip]
#[prelude_import]
use core::prelude::v1::*;
#[macro_use]
extern crate core;
#[macro_use]
extern crate compiler_builtins;

use core::ops::Generator;
use cortex_m_semihosting::{debug, hprintln};
use lm3s6965::Interrupt;
use panic_semihosting as _;
use rtfm::{Exclusive, Mutex};

#[allow(non_snake_case)]
fn init(_: init::Context) {
    // #[task(binds = GPIOA, resources = [shared2])]
    // // fn foo1(ctx: &'static mut foo1::Context) -> impl Generator<Yield = (), Return = !> {
    // fn foo1(mut ctx: foo1::Context) -> impl Generator<Yield = (), Return = !> {
    //     let mut local = 0;
    //     //let shared_res = Exclusive(ctx.resources.shared);
    //     // static shared_res : &mut u32 = ctx.resources.shared;
    //     // hprintln!("init foo1: {}", ctx.resources.shared);
    //     static mut X: u32 = 0;
    //     // let mut e = Exclusive(unsafe { &mut X });
    //     // //let mut e = Exclusive(ctx.resources.shared);
    //     // hprintln!(
    //     //     "ex {:?}",
    //     //     e.lock(|v| {
    //     //         *v += 1;
    //     //         *v
    //     //     })
    //     // )
    //     // .unwrap();
    //     hprintln!("{}", ctx.resources.shared2.lock(|v| *v));
    //     move || loop {
    //         hprintln!("foo1_1 {} {}", local, unsafe { X }).unwrap();
    //         // hprintln!(
    //         //     "ex {}",
    //         //     e.lock(|v| {
    //         //         *v += 1;
    //         //         *v
    //         //     })
    //         // )
    //         // .unwrap();

    //         local += 1;
    //         // shared_res.lock(|s| hprintln!("s {}", s));
    //         //   *shared_res += 1;
    //         // *ctx.resources.shared += 1;
    //         unsafe { X += 1 };
    //         yield;
    //         hprintln!("foo1_2 {} {}", local, unsafe { X }).unwrap();
    //         local += 1;
    //         // unsafe { X += 1 };
    //         yield;
    //     }
    // }

    // ctx.resources.shared2.lock(|v| *v);
    //        hprintln!("{}", ctx.resources.shared2.lock(|v| *v)).unwrap();

    ::cortex_m_semihosting::export::hstdout_str("init\n").unwrap();
    //
    rtfm::pend(Interrupt::GPIOA);
}
#[allow(non_snake_case)]
fn idle(_: idle::Context) -> ! {
    use rtfm::Mutex as _;
    ::cortex_m_semihosting::export::hstdout_str("idle\n").unwrap();
    rtfm::pend(Interrupt::GPIOA);
    ::cortex_m_semihosting::export::hstdout_str("idle1\n").unwrap();
    rtfm::pend(Interrupt::GPIOA);
    debug::exit(debug::EXIT_SUCCESS);
    loop {}
}
#[rustfmt::skip]
type Generatorfoo1= impl Generator<Yield = (), Return = !>;
static mut GENERATOR_FOO1: core::mem::MaybeUninit<Generatorfoo1> =
    core::mem::MaybeUninit::uninit();
#[allow(non_snake_case)]
fn foo1(mut ctx: foo1::Context) -> Generatorfoo1 {
    move || loop {
        ctx.resources.shared2.lock(|v| {
            *v += 1;
            hprintln!("foo1_1 lock      {}", v);
            rtfm::pend(Interrupt::GPIOB);
            hprintln!("foo1_1 unloclock {}", v);
        });
        ctx.resources.shared2.lock(|v| {
            *v += 1;
            hprintln!("foo1_1           {}", v);
        });

        yield;
        ctx.resources.shared2.lock(|v| {
            *v += 1;
            hprintln!("foo1_2           {}", v);
        });
        yield;
    }
}
#[allow(non_snake_case)]
fn foo2(mut ctx: foo2::Context) {
    use rtfm::Mutex as _;
    ctx.resources.shared2.lock(|v| {
        *v += 1;
        hprintln!("foo2             {}", v);
    });
}
#[allow(non_snake_case)]
fn foo3(mut ctx: foo3::Context) {
    use rtfm::Mutex as _;
    ctx.resources.shared2.lock(|v| *v += 1);
}
#[allow(non_snake_case)]
#[doc = "Initialization function"]
pub mod init {
    fn plepps() {}
    #[doc = r" Execution context"]
    pub struct Context {
        #[doc = r" Core (Cortex-M) peripherals"]
        pub core: rtfm::export::Peripherals,
    }
    impl Context {
        #[inline(always)]
        pub unsafe fn new(core: rtfm::export::Peripherals) -> Self {
            Context { core }
        }
    }
}
#[allow(non_snake_case)]
#[doc = "Idle loop"]
pub mod idle {
    fn plepps() {}
    #[doc = r" Execution context"]
    pub struct Context {}
    impl Context {
        #[inline(always)]
        pub unsafe fn new(priority: rtfm::export::Priority) -> Self {
            Context {}
        }
    }
}
mod resources {
    use rtfm::export::Priority;
    #[allow(non_camel_case_types)]
    pub struct shared2 {
        priority: Priority,
    }
    impl shared2 {
        #[inline(always)]
        pub unsafe fn new(priority: Priority) -> Self {
            shared2 { priority }
        }
        #[inline(always)]
        pub unsafe fn priority(&self) -> Priority {
            self.priority.clone()
        }
    }
}
#[allow(non_snake_case)]
#[doc = "Resources `foo1` has access to"]
pub struct foo1Resources {
    pub shared2: resources::shared2,
}
#[allow(non_snake_case)]
#[doc = "Hardware task"]
pub mod foo1 {
    fn plepps() {}
    #[doc(inline)]
    pub use super::foo1Resources as Resources;
    #[doc = r" Execution context"]
    pub struct Context {
        #[doc = r" Resources this task has access to"]
        pub resources: Resources,
    }
    impl<'a> Context {
        #[inline(always)]
        pub unsafe fn new(priority: rtfm::export::Priority) -> Self {
            Context {
                resources: Resources::new(priority),
            }
        }
    }
}
#[allow(non_snake_case)]
#[doc = "Resources `foo2` has access to"]
pub struct foo2Resources {
    pub shared2: resources::shared2,
}
#[allow(non_snake_case)]
#[doc = "Hardware task"]
pub mod foo2 {
    fn plepps() {}
    #[doc(inline)]
    pub use super::foo2Resources as Resources;
    #[doc = r" Execution context"]
    pub struct Context {
        #[doc = r" Resources this task has access to"]
        pub resources: Resources,
    }
    impl Context {
        #[inline(always)]
        pub unsafe fn new(priority: rtfm::export::Priority) -> Self {
            Context {
                resources: Resources::new(priority),
            }
        }
    }
}
#[allow(non_snake_case)]
#[doc = "Resources `foo3` has access to"]
pub struct foo3Resources {
    pub shared2: resources::shared2,
}
#[allow(non_snake_case)]
#[doc = "Hardware task"]
pub mod foo3 {
    fn plepps() {}
    #[doc(inline)]
    pub use super::foo3Resources as Resources;
    #[doc = r" Execution context"]
    pub struct Context {
        #[doc = r" Resources this task has access to"]
        pub resources: Resources,
    }
    impl Context {
        #[inline(always)]
        pub unsafe fn new(priority: rtfm::export::Priority) -> Self {
            Context {
                resources: Resources::new(priority),
            }
        }
    }
}
#[doc = r" Implementation details"]
const APP: () = {
    #[doc = r" Always include the device crate which contains the vector table"]
    use lm3s6965 as _;
    #[allow(non_upper_case_globals)]
    static mut shared2: u32 = 1;
    impl<'a> rtfm::Mutex for resources::shared2 {
        type T = u32;
        #[inline(always)]
        fn lock<R>(&mut self, f: impl FnOnce(&mut u32) -> R) -> R {
            #[doc = r" Priority ceiling"]
            const CEILING: u8 = 3u8;
            unsafe {
                rtfm::export::lock(
                    &mut shared2,
                    &self.priority(),
                    CEILING,
                    lm3s6965::NVIC_PRIO_BITS,
                    f,
                )
            }
        }
    }
    #[allow(non_snake_case)]
    #[no_mangle]
    unsafe fn GPIOA() {
        const PRIORITY: u8 = 1u8;
        rtfm::export::run(PRIORITY, || {
            ::cortex_m_semihosting::export::hstdout_str("interrupt dispatch\n")
                .unwrap();
            let mut g: &mut dyn Generator<Yield = (), Return = !> =
                &mut *GENERATOR_FOO1.as_mut_ptr();
            core::pin::Pin::new_unchecked(&mut *g).resume();
        });
    }
    impl foo1Resources {
        #[inline(always)]
        unsafe fn new(priority: rtfm::export::Priority) -> Self {
            foo1Resources {
                shared2: resources::shared2::new(priority),
            }
        }
    }
    #[allow(non_snake_case)]
    #[no_mangle]
    unsafe fn GPIOB() {
        const PRIORITY: u8 = 2u8;
        rtfm::export::run(PRIORITY, || {
            crate::foo2(foo2::Context::new(rtfm::export::Priority::new(
                PRIORITY,
            )))
        });
    }
    impl<'a> foo2Resources {
        #[inline(always)]
        unsafe fn new(priority: rtfm::export::Priority) -> Self {
            foo2Resources {
                shared2: resources::shared2::new(priority),
            }
        }
    }
    #[allow(non_snake_case)]
    #[no_mangle]
    unsafe fn GPIOC() {
        const PRIORITY: u8 = 3u8;
        rtfm::export::run(PRIORITY, || {
            crate::foo3(foo3::Context::new(rtfm::export::Priority::new(
                PRIORITY,
            )))
        });
    }
    impl foo3Resources {
        #[inline(always)]
        unsafe fn new(priority: rtfm::export::Priority) -> Self {
            foo3Resources {
                shared2: resources::shared2::new(priority),
            }
        }
    }
    #[no_mangle]
    unsafe extern "C" fn main() -> ! {
        rtfm::export::interrupt::disable();
        let mut core: rtfm::export::Peripherals = core::mem::transmute(());
        let _ = [(); ((1 << lm3s6965::NVIC_PRIO_BITS) - 1u8 as usize)];
        core.NVIC.set_priority(
            lm3s6965::Interrupt::GPIOA,
            rtfm::export::logical2hw(1u8, lm3s6965::NVIC_PRIO_BITS),
        );
        rtfm::export::NVIC::unmask(lm3s6965::Interrupt::GPIOA);
        let _ = [(); ((1 << lm3s6965::NVIC_PRIO_BITS) - 2u8 as usize)];
        core.NVIC.set_priority(
            lm3s6965::Interrupt::GPIOB,
            rtfm::export::logical2hw(2u8, lm3s6965::NVIC_PRIO_BITS),
        );
        rtfm::export::NVIC::unmask(lm3s6965::Interrupt::GPIOB);
        let _ = [(); ((1 << lm3s6965::NVIC_PRIO_BITS) - 3u8 as usize)];
        core.NVIC.set_priority(
            lm3s6965::Interrupt::GPIOC,
            rtfm::export::logical2hw(3u8, lm3s6965::NVIC_PRIO_BITS),
        );
        rtfm::export::NVIC::unmask(lm3s6965::Interrupt::GPIOC);
        let late = init(init::Context::new(core.into()));
        const PRIORITY: u8 = 1u8;
        unsafe {
            GENERATOR_FOO1.as_mut_ptr().write(foo1(foo1::Context::new(
                rtfm::export::Priority::new(PRIORITY),
            )));
        };
        rtfm::export::interrupt::enable();
        idle(idle::Context::new(rtfm::export::Priority::new(0)))
    }
};
