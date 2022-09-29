#![feature(prelude_import)]
//! examples/hardware.rs
#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]
#[prelude_import]
use core::prelude::rust_2021::*;
#[macro_use]
extern crate core;
#[macro_use]
extern crate compiler_builtins;
use panic_semihosting as _;
/// The RTIC application module
pub mod app {
    /// Always include the device crate which contains the vector table
    use lm3s6965 as you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml;
    use cortex_m_semihosting::{debug, hprintln};
    use lm3s6965::Interrupt;
    /// User code from within the module
    /// User code end
    #[inline(always)]
    #[allow(non_snake_case)]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        rtic::pend(Interrupt::UART0);
        ::cortex_m_semihosting::export::hstdout_str("init\n").unwrap();
        (Shared {}, Local {}, init::Monotonics())
    }
    #[allow(non_snake_case)]
    fn foo(_: foo::Context) {
        use rtic::Mutex as _;
        use rtic::mutex::prelude::*;
        ::cortex_m_semihosting::export::hstdout_str("foo\n").unwrap();
        debug::exit(debug::EXIT_SUCCESS);
    }
    struct Shared {}
    struct Local {}
    /// Monotonics used by the system
    #[allow(non_snake_case)]
    #[allow(non_camel_case_types)]
    pub struct __rtic_internal_Monotonics();
    /// Execution context
    #[allow(non_snake_case)]
    #[allow(non_camel_case_types)]
    pub struct __rtic_internal_init_Context<'a> {
        /// Core (Cortex-M) peripherals
        pub core: rtic::export::Peripherals,
        /// Device peripherals
        pub device: lm3s6965::Peripherals,
        /// Critical section token for init
        pub cs: rtic::export::CriticalSection<'a>,
    }
    impl<'a> __rtic_internal_init_Context<'a> {
        #[inline(always)]
        pub unsafe fn new(core: rtic::export::Peripherals) -> Self {
            __rtic_internal_init_Context {
                device: lm3s6965::Peripherals::steal(),
                cs: rtic::export::CriticalSection::new(),
                core,
            }
        }
    }
    #[allow(non_snake_case)]
    ///Initialization function
    pub mod init {
        pub use super::__rtic_internal_Monotonics as Monotonics;
        pub use super::__rtic_internal_init_Context as Context;
    }
    /// Execution context
    #[allow(non_snake_case)]
    #[allow(non_camel_case_types)]
    pub struct __rtic_internal_foo_Context {}
    impl __rtic_internal_foo_Context {
        #[inline(always)]
        pub unsafe fn new(priority: &rtic::export::Priority) -> Self {
            __rtic_internal_foo_Context {}
        }
    }
    #[allow(non_snake_case)]
    ///Hardware task
    pub mod foo {
        pub use super::__rtic_internal_foo_Context as Context;
    }
    /// app module
    #[doc(hidden)]
    #[allow(non_upper_case_globals)]
    const __rtic_internal_MASK_CHUNKS: usize = rtic::export::compute_mask_chunks([
        lm3s6965::Interrupt::UART0 as u32,
    ]);
    #[doc(hidden)]
    #[allow(non_upper_case_globals)]
    const __rtic_internal_MASKS: [rtic::export::Mask<__rtic_internal_MASK_CHUNKS>; 3] = [
        rtic::export::create_mask([lm3s6965::Interrupt::UART0 as u32]),
        rtic::export::create_mask([]),
        rtic::export::create_mask([]),
    ];
    #[allow(non_snake_case)]
    #[no_mangle]
    unsafe fn UART0() {
        const PRIORITY: u8 = 1u8;
        rtic::export::run(
            PRIORITY,
            || { foo(foo::Context::new(&rtic::export::Priority::new(PRIORITY))) },
        );
    }
    #[doc(hidden)]
    mod rtic_ext {
        use super::*;
        #[no_mangle]
        unsafe extern "C" fn main() -> ! {
            const _CONST_CHECK: () = {
                if !rtic::export::have_basepri() {
                    if (lm3s6965::Interrupt::UART0 as usize)
                        >= (__rtic_internal_MASK_CHUNKS * 32)
                    {
                        ::core::panicking::panic_fmt(
                            ::core::fmt::Arguments::new_v1(
                                &[
                                    "An interrupt out of range is used while in armv6 or armv8m.base",
                                ],
                                &[],
                            ),
                        );
                    }
                } else {}
            };
            let _ = _CONST_CHECK;
            rtic::export::interrupt::disable();
            let mut core: rtic::export::Peripherals = rtic::export::Peripherals::steal()
                .into();
            const _: () = if (1 << lm3s6965::NVIC_PRIO_BITS) < 1u8 as usize {
                ::core::panicking::panic_fmt(
                    ::core::fmt::Arguments::new_v1(
                        &[
                            "Maximum priority used by interrupt vector \'UART0\' is more than supported by hardware",
                        ],
                        &[],
                    ),
                );
            };
            core.NVIC
                .set_priority(
                    you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml::interrupt::UART0,
                    rtic::export::logical2hw(1u8, lm3s6965::NVIC_PRIO_BITS),
                );
            rtic::export::NVIC::unmask(
                you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml::interrupt::UART0,
            );
            #[inline(never)]
            fn __rtic_init_resources<F>(f: F)
            where
                F: FnOnce(),
            {
                f();
            }
            __rtic_init_resources(|| {
                let (shared_resources, local_resources, mut monotonics) = init(
                    init::Context::new(core.into()),
                );
                rtic::export::interrupt::enable();
            });
            loop {
                rtic::export::nop()
            }
        }
    }
}
