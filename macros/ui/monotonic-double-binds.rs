#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[monotonic(binds = Tim1, binds = Tim2)]
    type Fast = hal::Tim1Monotonic;
}
