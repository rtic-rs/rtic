#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[monotonic(binds = Tim1)]
    type Fast1 = hal::Tim1Monotonic;

    #[monotonic(binds = Tim2)]
    type Fast1 = hal::Tim2Monotonic;
}
