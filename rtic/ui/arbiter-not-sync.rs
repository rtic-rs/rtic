#![no_main]

#[rtic::app(device = lm3s6965)]
mod app {
    use rtic_sync::arbiter::Arbiter;
    use std::rc::Rc;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    static A: Arbiter<Option<Rc<u32>>> = Arbiter::new(None);

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        A.try_access().unwrap().replace(Rc::new(0));
        (Shared {}, Local {})
    }

    #[task(binds = QEI0, priority = 1)]
    fn p1(_: p1::Context) {
        let c: Rc<u32> = loop {
            if let Some(e) = A.try_access()
                && let Some(c) = &*e
            {
                // First clone taken with lock.
                break Rc::clone(c);
            };
        };
        loop {
            // Subsequent clones without lock.
            // Non-atomic updates on reference
            // count across tasks.
            let _ = Rc::clone(&c);
        }
    }

    #[task(binds = SSI0, priority = 2)]
    fn p2(_: p2::Context) {
        let c: Rc<u32> = loop {
            if let Some(e) = A.try_access()
                && let Some(c) = &*e
            {
                break Rc::clone(c);
            };
        };
        loop {
            let _ = Rc::clone(&c);
        }
    }
}
