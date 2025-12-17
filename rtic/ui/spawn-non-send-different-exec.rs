#![no_main]

#[rtic::app(device = lm3s6965, dispatchers = [SSI0, GPIOA])]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_cx: init::Context) -> (Shared, Local) {
        (Shared {}, Local {})
    }

    #[task(priority = 1)]
    async fn foo(_cx: foo::Context, _nsns: NotSendNotSync) {}

    #[task(priority = 2)]
    async fn bar(cx: bar::Context) {
        cx.local_spawner.foo(Default::default()).ok();
    }
}

#[derive(Default, Debug)]
struct NotSendNotSync(core::marker::PhantomData<*mut u8>);
