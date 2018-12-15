#![no_main]
#![no_std]

extern crate lm3s6965;
extern crate panic_halt;
extern crate rtfm;

use core::marker::PhantomData;

use rtfm::app;

pub struct NotSend {
    _0: PhantomData<*const ()>,
}

#[app(device = lm3s6965)]
const APP: () = {
    static mut X: NotSend = ();
    static mut Y: Option<NotSend> = None;

    #[init(resources = [Y])]
    fn init() {
        *resources.Y = Some(NotSend { _0: PhantomData });

        X = NotSend { _0: PhantomData };
    }

    #[idle(resources = [X, Y])]
    fn idle() -> ! {
        loop {}
    }
};
