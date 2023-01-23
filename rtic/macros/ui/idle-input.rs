#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[idle]
    fn idle(_: idle::Context, _undef: u32) -> ! {
        loop {}
    }
}
