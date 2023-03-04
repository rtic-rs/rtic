#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[init]
    unsafe fn init(_: init::Context) -> (Shared, Local) {}
}
