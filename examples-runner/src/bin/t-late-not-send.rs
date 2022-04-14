//! [compile-pass] shared resources don't need to be `Send` if they are owned by `idle`

#![no_main]
#![no_std]

use core::marker::PhantomData;

use examples_runner as _;

pub struct NotSend {
    _0: PhantomData<*const ()>,
}

#[rtic::app(device = examples_runner::pac)]
mod app {
    use super::NotSend;
    use core::marker::PhantomData;
    use examples_runner::exit;

    #[shared]
    struct Shared {
        x: NotSend,
        y: Option<NotSend>,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        (
            Shared {
                x: NotSend { _0: PhantomData },
                y: None,
            },
            Local {},
            init::Monotonics(),
        )
    }

    #[idle(shared = [x, y])]
    fn idle(_: idle::Context) -> ! {
        exit();
        // loop {
        //     cortex_m::asm::nop();
        // }
    }
}
