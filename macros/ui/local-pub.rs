#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        pub x: u32,
    }

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {}
}
