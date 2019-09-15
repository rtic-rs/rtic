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
    struct Resources {
        x: NotSend,
        #[init(None)]
        y: Option<NotSend>,
    }

    #[init(resources = [y])]
    fn init(c: init::Context) -> init::LateResources {
        // equivalent to late resource initialization
        *c.resources.y = Some(NotSend { _0: PhantomData });

        init::LateResources {
            x: NotSend { _0: PhantomData },
        }
    }

    #[idle(resources = [x, y])]
    fn idle(_: idle::Context) -> ! {
        loop {}
    }
};
