#![no_main]

#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_cx: init::Context) -> (Shared, Local) {
        foo::spawn(NotSendNotSync::default()).ok();
        (Shared {}, Local {})
    }

    #[task(priority = 1)]
    async fn foo(_cx: foo::Context, _nsns: NotSendNotSync) {}
}

#[derive(Default, Debug)]
struct NotSendNotSync(core::marker::PhantomData<*mut u8>);
