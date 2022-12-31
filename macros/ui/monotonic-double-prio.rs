#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[monotonic(binds = Tim1, priority = 1, priority = 2)]
    type Fast = hal::Tim1Monotonic;
}
