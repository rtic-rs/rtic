#![no_main]

#[rtic::app(device = lm3s6965, dispatchers = [SSI0, QEI0, GPIOA])]
mod app {
    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        medium_prio::spawn().unwrap();
        (Shared {}, Local {})
    }

    #[task(priority = 2)]
    async fn medium_prio(_: medium_prio::Context) {
        let mut will_be_dropped: usize = 41;
        will_be_dropped += 1;

        high_prio_print::spawn(&mut will_be_dropped).unwrap();
    }

    #[task(priority = 3)]
    async fn high_prio_print(_: high_prio_print::Context, mut_ref: &mut usize) {
        *mut_ref += 1000;
    }
}
