#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[monotonic(binds = Tim1, default = true, default = false)]
    type Fast = hal::Tim1Monotonic;
}
