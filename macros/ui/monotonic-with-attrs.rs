#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[no_mangle]
    #[monotonic(binds = Tim1)]
    type Fast = hal::Tim1Monotonic;
}
