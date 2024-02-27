//! [compile-pass] shared resources don't need to be `Send` if they are owned by `idle`

#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(unsafe_code)]
#![deny(missing_docs)]

use core::marker::PhantomData;
use panic_semihosting as _;

/// Not send
pub struct NotSend {
    _0: PhantomData<*const ()>,
}

#[rtic::app(device = lm3s6965)]
mod app {
    use super::NotSend;
    use core::marker::PhantomData;
    use cortex_m_semihosting::debug;

    #[shared]
    struct Shared {
        x: NotSend,
        y: Option<NotSend>,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        (
            Shared {
                x: NotSend { _0: PhantomData },
                y: None,
            },
            Local {},
        )
    }

    #[idle(shared = [x, y])]
    fn idle(_: idle::Context) -> ! {
        debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator
        loop {
            cortex_m::asm::nop();
        }
    }
}
