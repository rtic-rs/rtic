//! examples/big-struct-opt.rs
//!
//! Example on how to initialize a large struct without needing to copy it via `LateResources`,
//! effectively saving stack space needed for the copies.

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(missing_docs)]
#![feature(type_alias_impl_trait)]

use panic_semihosting as _;

/// Some big struct
pub struct BigStruct {
    /// Big content
    pub data: [u8; 2048],
}

impl BigStruct {
    fn new() -> Self {
        BigStruct { data: [22; 2048] }
    }
}

#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    use super::BigStruct;
    use core::mem::MaybeUninit;
    use cortex_m_semihosting::{debug, hprintln};
    use lm3s6965::Interrupt;

    #[shared]
    struct Shared {
        big_struct: &'static mut BigStruct,
    }

    #[local]
    struct Local {}

    #[init(local = [bs: MaybeUninit<BigStruct> = MaybeUninit::uninit()])]
    fn init(cx: init::Context) -> (Shared, Local) {
        let big_struct = unsafe {
            // write directly into the static storage
            cx.local.bs.as_mut_ptr().write(BigStruct::new());
            &mut *cx.local.bs.as_mut_ptr()
        };

        rtic::pend(Interrupt::UART0);
        async_task::spawn().unwrap();
        (
            Shared {
                // assign the reference so we can use the resource
                big_struct,
            },
            Local {},
        )
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            hprintln!("idle");
            debug::exit(debug::EXIT_SUCCESS);
        }
    }

    #[task(binds = UART0, shared = [big_struct])]
    fn uart0(mut cx: uart0::Context) {
        cx.shared
            .big_struct
            .lock(|b| hprintln!("uart0 data:{:?}", &b.data[0..5]));
    }

    #[task(shared = [big_struct], priority = 2)]
    async fn async_task(mut cx: async_task::Context) {
        cx.shared
            .big_struct
            .lock(|b| hprintln!("async_task data:{:?}", &b.data[0..5]));
    }
}
