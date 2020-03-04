init Ident { ident: "shared", span: #0 bytes(309..315) }
-- exclusive -- pos 0
before attrs [Attribute { pound_token: Pound, style: Outer, bracket_token: Bracket, path: Path { leading_colon: None, segments: [PathSegment { ident: Ident { ident: "exclusive", span: #0 bytes(332..341) }, arguments: None }] }, tokens: TokenStream [] }]
after remove attrs []
#![feature(prelude_import)]
//! examples/local.rs

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]
#[prelude_import]
use core::prelude::v1::*;
#[macro_use]
extern crate core;
#[macro_use]
extern crate compiler_builtins;

use cortex_m_semihosting::{debug, hprintln};
use lm3s6965::Interrupt;
use panic_semihosting as _;

#[allow(non_snake_case)]
fn init(
    // A resource
    // #[init(7)]
    _: init::Context,
) -> init::LateResources {
    rtfm::pend(Interrupt::UART0);
    rtfm::pend(Interrupt::UART1);
    init::LateResources { ex: 0 }
}
#[allow(non_snake_case)]
fn idle(
    // `shared` cannot be accessed from this context
    _cx: idle::Context,
) -> ! {
    use rtfm::Mutex as _;
    debug::exit(debug::EXIT_SUCCESS);

    // error: no `resources` field in `idle::Context`
    // _cx.resources.shared += 1;

    loop {}
}
#[allow(non_snake_case)]
fn uart0(
    // `shared` can be accessed from this context
    cx: uart0::Context,
) {
    use rtfm::Mutex as _;
    let shared: &mut u32 = cx.resources.shared;
    *shared += 1;

    // // `shared` can be accessed from this context
    // #[task(binds = UART1, resources = [shared])]
    // fn uart1(cx: uart1::Context) {
    //     *cx.resources.shared += 1;

    //     hprintln!("UART1: shared = {}", cx.resources.shared).unwrap();
    // }
    ::cortex_m_semihosting::export::hstdout_fmt(::core::fmt::Arguments::new_v1(
        &["UART0: shared = ", "\n"],
        &match (&shared,) {
            (arg0,) => [::core::fmt::ArgumentV1::new(
                arg0,
                ::core::fmt::Display::fmt,
            )],
        },
    ))
    .unwrap();
}
#[doc = r" Resources initialized at runtime"]
#[allow(non_snake_case)]
pub struct initLateResources {
    pub ex: u32,
}
#[allow(non_snake_case)]
#[doc = "Initialization function"]
pub mod init {
    #[doc(inline)]
    pub use super::initLateResources as LateResources;
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
    #[doc = r" Execution context"]
    pub struct Context {}
    impl Context {
        #[inline(always)]
        pub unsafe fn new(priority: &rtfm::export::Priority) -> Self {
            Context {}
        }
    }
}
#[allow(non_snake_case)]
#[doc = "Resources `uart0` has access to"]
pub struct uart0Resources<'a> {
    pub shared: &'a mut u32,
    pub ex: &'a mut u32,
}
#[allow(non_snake_case)]
#[doc = "Hardware task"]
pub mod uart0 {
    #[doc(inline)]
    pub use super::uart0Resources as Resources;
    #[doc = r" Execution context"]
    pub struct Context<'a> {
        #[doc = r" Resources this task has access to"]
        pub resources: Resources<'a>,
    }
    impl<'a> Context<'a> {
        #[inline(always)]
        pub unsafe fn new(priority: &'a rtfm::export::Priority) -> Self {
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
    static mut shared: u32 = 0;
    #[allow(non_upper_case_globals)]
    #[link_section = ".uninit.rtfm0"]
    #[exclusive]
    static mut ex: core::mem::MaybeUninit<u32> = core::mem::MaybeUninit::uninit();
    #[allow(non_snake_case)]
    #[no_mangle]
    unsafe fn UART0() {
        const PRIORITY: u8 = 1u8;
        rtfm::export::run(PRIORITY, || {
            crate::uart0(uart0::Context::new(&rtfm::export::Priority::new(PRIORITY)))
        });
    }
    impl<'a> uart0Resources<'a> {
        #[inline(always)]
        unsafe fn new(priority: &'a rtfm::export::Priority) -> Self {
            uart0Resources {
                shared: &mut shared,
                ex: &mut *ex.as_mut_ptr(),
            }
        }
    }
    #[no_mangle]
    unsafe extern "C" fn main() -> ! {
        rtfm::export::assert_send::<u32>();
        rtfm::export::interrupt::disable();
        let mut core: rtfm::export::Peripherals = core::mem::transmute(());
        let _ = [(); ((1 << lm3s6965::NVIC_PRIO_BITS) - 1u8 as usize)];
        core.NVIC.set_priority(
            lm3s6965::Interrupt::UART0,
            rtfm::export::logical2hw(1u8, lm3s6965::NVIC_PRIO_BITS),
        );
        rtfm::export::NVIC::unmask(lm3s6965::Interrupt::UART0);
        let late = init(init::Context::new(core.into()));
        ex.as_mut_ptr().write(late.ex);
        rtfm::export::interrupt::enable();
        idle(idle::Context::new(&rtfm::export::Priority::new(0)))
    }
};
