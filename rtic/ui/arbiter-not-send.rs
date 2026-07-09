#![no_main]

#[rtic::app(device = lm3s6965, dispatchers = [SSI0, QEI0])]
mod app {
    use rtic_sync::arbiter::Arbiter;
    use std::rc::Rc;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        x: Arbiter<Rc<u32>>,
        y: Arbiter<Rc<u32>>,
    }

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        p1::spawn().unwrap();
        p2::spawn().unwrap();

        let x = Rc::new(5);
        let y = Rc::clone(&x);
        (
            Shared {},
            Local {
                x: Arbiter::new(x),
                y: Arbiter::new(y),
            },
        )
    }

    #[task(priority = 1, local = [x])]
    async fn p1(cx: p1::Context) -> ! {
        // Non-atomic update of reference count
        // through two different Arbiters.
        let c: Rc<u32> = Rc::clone(&*cx.local.x.access().await);
        loop {
            let _ = Rc::clone(&c);
        }
    }

    #[task(priority = 2, local = [y])]
    async fn p2(cx: p2::Context) {
        let c: Rc<u32> = Rc::clone(&*cx.local.y.access().await);
        loop {
            let _ = Rc::clone(&c);
        }
    }
}
