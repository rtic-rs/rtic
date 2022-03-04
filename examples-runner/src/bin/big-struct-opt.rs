//! examples/big-struct-opt.rs
//!
//! Example on how to initialize a large struct without needing to copy it via `LateResources`,
//! effectively saving stack space needed for the copies.

#![no_main]
#![no_std]

use examples_runner as _;

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

#[rtic::app(device = examples_runner::pac)]
mod app {
    use super::BigStruct;
    use core::mem::MaybeUninit;
    use examples_runner::exit;

    #[shared]
    struct Shared {
        big_struct: &'static mut BigStruct,
    }

    #[local]
    struct Local {}

    #[init(local = [bs: MaybeUninit<BigStruct> = MaybeUninit::uninit()])]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        let big_struct = unsafe {
            // write directly into the static storage
            cx.local.bs.as_mut_ptr().write(BigStruct::new());
            &mut *cx.local.bs.as_mut_ptr()
        };

        (
            Shared {
                // assign the reference so we can use the resource
                big_struct,
            },
            Local {},
            init::Monotonics(),
        )
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        exit();
    }

    #[task(binds = UART0, shared = [big_struct])]
    fn task(_: task::Context) {}
}
