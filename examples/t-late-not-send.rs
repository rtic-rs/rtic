//! [compile-pass] late resources don't need to be `Send` if they are owned by `idle`

#![no_main]
#![no_std]

use core::marker::PhantomData;

use panic_halt as _;

pub struct NotSend {
    _0: PhantomData<*const ()>,
}

#[rtic::app(device = lm3s6965)]
mod app {
    use super::NotSend;
    use core::marker::PhantomData;

    #[resources]
    struct Resources {
        x: NotSend,
        #[init(None)]
        y: Option<NotSend>,
    }

    #[init]
    fn init(_: init::Context) -> (init::LateResources, init::Monotonics) {
        (
            init::LateResources {
                x: NotSend { _0: PhantomData },
            },
            init::Monotonics(),
        )
    }

    #[idle(resources = [x, y])]
    fn idle(_: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }
}
