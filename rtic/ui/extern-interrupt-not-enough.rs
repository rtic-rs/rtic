#![no_main]

#[rtic::app(device = lm3s6965)]
mod app {
    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        (Shared {}, Local {})
    }

    #[task(priority = 1)]
    async fn a(_: a::Context) {}
}
