#![no_main]

#[rtic_macros::mock_app(device = mock, dispatchers = [EXTI0])]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_cx: init::Context) -> (Shared, Local) {
        task1::spawn().ok();
        (Shared {}, Local {})
    }

    #[task(priority = 1)]
    async fn task1(cx: task1::Context) {
        cx.local_spawner.task2(Default::default()).ok();
    }

    #[task(priority = 1, is_local_task = true)]
    async fn task2(_cx: task2::Context, _nsns: super::NotSendNotSync) {}
}

#[derive(Default)]
struct NotSendNotSync(PhantomData<*mut u8>);