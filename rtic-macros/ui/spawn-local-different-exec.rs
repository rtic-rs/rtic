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
        (Shared {}, Local {})
    }

    #[task(priority = 1, local_task)]
    async fn foo(_cx: foo::Context) {}

    #[task(priority = 2)]
    async fn bar(cx: bar::Context) {
        cx.local_spawner.foo().ok();
    }
}
