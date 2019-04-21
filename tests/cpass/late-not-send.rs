#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate lm3s6965;
extern crate panic_halt;
extern crate rtfm;

use core::marker::PhantomData;

pub struct NotSend {
    _0: PhantomData<*const ()>,
}

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    static mut X: NotSend = ();
    static mut Y: Option<NotSend> = None;

    #[init(resources = [Y])]
    fn init(c: init::Context) -> init::LateResources {
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
