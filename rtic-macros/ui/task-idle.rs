#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[idle]
    fn foo(_: foo::Context) -> ! {
        loop {}
    }

    // name collides with `#[idle]` function
    #[task]
    async fn foo(_: foo::Context) {}
}
