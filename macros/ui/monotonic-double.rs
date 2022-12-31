#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[monotonic(binds = Tim1)]
    type Fast = hal::Tim1Monotonic;

    #[monotonic(binds = Tim1)]
    type Fast = hal::Tim1Monotonic;
}
