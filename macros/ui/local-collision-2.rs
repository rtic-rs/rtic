#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        a: u32,
    }

    #[task(local = [a: u8 = 3])]
    async fn bar(_: bar::Context) {}

    #[init(local = [a: u16 = 2])]
    fn init(_: init::Context) -> (Shared, Local) {}
}
