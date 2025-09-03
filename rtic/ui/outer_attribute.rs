#![no_main]
#![deny(unfulfilled_lint_expectations)]

#[rtic::app(device = lm3s6965)]
#[expect(while_true)]
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
