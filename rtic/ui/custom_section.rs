#![no_main]

compile_error!("This should be the only error in the file");

#[rtic::app(device = lm3s6965)]
mod app {
    #[shared]
    struct Shared {
        #[unsafe(link_section = ".custom_section")]
        foo: (),
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_cx: init::Context) -> (Shared, Local) {
        (Shared { foo: () }, Local {})
    }
}
