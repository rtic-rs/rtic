#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[idle(local = [A], local = [B])]
    fn idle(_: idle::Context) -> ! {
        loop {}
    }
}
