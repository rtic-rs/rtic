#![no_main]
#![deny(unfulfilled_lint_expectations)]

#[rtic::app(device = lm3s6965)]
mod app {
    #![expect(while_true)]
    
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
