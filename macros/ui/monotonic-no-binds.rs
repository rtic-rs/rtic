#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[monotonic()]
    type Fast = hal::Tim1Monotonic;
}
