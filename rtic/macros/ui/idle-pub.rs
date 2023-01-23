#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[idle]
    pub fn idle(_: idle::Context) -> ! {
        loop {}
    }
}
