#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[local]
    struct Local {
        pub x: u32,
    }
}
