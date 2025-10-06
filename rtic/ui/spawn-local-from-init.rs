#![no_main]

#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_cx: init::Context) -> (Shared, Local) {
        foo::spawn().ok();
        (Shared {}, Local {})
    }

    #[task(priority = 1, local_task)]
    async fn foo(_cx: foo::Context) {}
}
