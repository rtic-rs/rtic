#![no_main]

#[rtic::app(device = lm3s6965, dispatchers = [GPIOA])]
mod app {
    #[shared]
    struct Shared {
        a: u32,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        (Shared { a: 0 }, Local {}, init::Monotonics())
    }

    #[task(shared = [a])]
    fn foo(mut c: foo::Context) {
        c.shared.lock(|_| {
            c.shared.a.lock(|_| {}); // borrow error
        });
    }
}
