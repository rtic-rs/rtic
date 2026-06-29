#![no_main]

#[rtic::app(device = lm3s6965)]
mod app {
    use core::cell::RefCell;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        x: &'static RefCell<u32>,
        y: &'static RefCell<u32>,
    }

    #[init(local = [xy: RefCell<u32> = RefCell::new(0)])]
    fn init(cx: init::Context) -> (Shared, Local) {
        (
            Shared {},
            Local {
                x: cx.local.xy,
                y: cx.local.xy,
            },
        )
    }

    #[task(binds = SSI0, priority = 1, local = [x])]
    fn p1(cx: p1::Context) {
        *cx.local.x.borrow_mut() += 1;
    }

    #[task(binds = QEI0, priority = 2, local = [y])]
    fn p2(cx: p2::Context) {
        *cx.local.y.borrow_mut() += 1;
    }
}
