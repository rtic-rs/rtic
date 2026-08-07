#![no_main]

#[rtic_macros::mock_app(device = mock, dispatchers = [EXTI0])]
mod app {
    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        todo!()
    }

    extern "Rust" {
        #[task]
        async fn foo(_: foo::Context<'static>);
    }
}
