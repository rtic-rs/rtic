#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[init]
    fn init(_: init::Context) -> u32 {
        0
    }
}
