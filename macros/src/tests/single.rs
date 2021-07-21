use quote::quote;
use rtic_syntax::Settings;

#[test]
fn analyze() {
    let mut settings = Settings::default();
    settings.parse_extern_interrupt = true;
    let (app, analysis) = rtic_syntax::parse2(
        // First interrupt is assigned to the highest priority dispatcher
        quote!(device = pac, dispatchers = [B, A]),
        quote!(
            mod app {
                #[shared]
                struct Shared {}

                #[local]
                struct Local {}

                #[init]
                fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
                    (Shared {}, Local {}, init::Monotonics())
                }

                #[task(priority = 1)]
                fn a(_: a::Context) {}

                #[task(priority = 2)]
                fn b(_: b::Context) {}
            }
        ),
        settings,
    )
    .unwrap();

    let analysis = crate::analyze::app(analysis, &app);
    let interrupts = &analysis.interrupts;
    assert_eq!(interrupts.len(), 2);
    assert_eq!(interrupts[&2].0.to_string(), "B");
    assert_eq!(interrupts[&1].0.to_string(), "A");
}
