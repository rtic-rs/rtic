#![no_main]

#[rtic::app(device = lm3s6965, dispatchers = [QEI0])]
mod app {
    use rtic_sync::signal::{Signal, SignalReader, SignalWriter};
    use std::cell::RefCell;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        r: SignalReader<'static, CopyNotSend>,
        w: SignalWriter<'static, CopyNotSend>,
    }

    type CopyNotSend = Option<&'static RefCell<u32>>;

    #[init(local = [s: Signal<CopyNotSend> = Signal::new()])]
    fn init(cx: init::Context) -> (Shared, Local) {
        let (w, r) = cx.local.s.split();
        (Shared {}, Local { r, w })
    }

    #[idle(local = [w, c: RefCell<u32> = RefCell::new(0)])]
    fn idle(cx: idle::Context) -> ! {
        // Copy c into the signal, making it
        // accessible to another task / thread.
        let c: &'static RefCell<u32> = cx.local.c;
        cx.local.w.write(Some(c));

        loop {
            let mut b = c.borrow_mut();

            // Exclusive borrow in one task.
            let x: &mut u32 = &mut *b;
            *x += 1;
            p2::spawn().unwrap();
            *x += 1;
        }
    }

    #[task(priority = 2, local = [r])]
    async fn p2(cx: p2::Context) {
        let c = cx.local.r.wait().await.unwrap();
        let mut b = c.borrow_mut();

        // Exclusive borrow in another task
        // without proper locking.
        let x: &mut u32 = &mut *b;
        *x += 1;
    }
}
