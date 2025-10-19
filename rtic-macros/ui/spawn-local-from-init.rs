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
        foo::spawn().ok();
        (Shared {}, Local {})
    }

    #[task(priority = 1, is_local_task = true)]
    async fn foo(_cx: foo::Context) {}
}
