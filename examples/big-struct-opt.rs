//! examples/big-struct-opt.rs
//!
//! Example on how to initialize a large struct without needing to copy it via `LateResources`,
//! effectively saving stack space needed for the copies.

#![no_main]
#![no_std]

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

#[rtic::app(device = lm3s6965)]
mod app {
    use super::BigStruct;
    use core::mem::MaybeUninit;

    #[resources]
    struct Resources {
        big_struct: &'static mut BigStruct,
    }

    #[init]
    fn init(_: init::Context) -> (init::LateResources, init::Monotonics) {
        let big_struct = unsafe {
            static mut BIG_STRUCT: MaybeUninit<BigStruct> = MaybeUninit::uninit();

            // write directly into the static storage
            BIG_STRUCT.as_mut_ptr().write(BigStruct::new());
            &mut *BIG_STRUCT.as_mut_ptr()
        };

        (
            init::LateResources {
                // assign the reference so we can use the resource
                big_struct,
            },
            init::Monotonics(),
        )
    }
}
