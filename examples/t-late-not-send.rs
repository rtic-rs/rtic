//! [compile-pass] late resources don't need to be `Send` if they are owned by `idle`

#![no_main]
#![no_std]

use core::marker::PhantomData;

use panic_halt as _;

pub struct NotSend {
    _0: PhantomData<*const ()>,
}

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    extern "C" {
        static mut X: NotSend;
    }

    static mut Y: Option<NotSend> = None;

    #[init(resources = [Y])]
    fn init(c: init::Context) -> init::LateResources {
        // equivalent to late resource initialization
        *c.resources.Y = Some(NotSend { _0: PhantomData });

        init::LateResources {
            X: NotSend { _0: PhantomData },
        }
    }

    #[idle(resources = [X, Y])]
    fn idle(_: idle::Context) -> ! {
        loop {}
    }
};
