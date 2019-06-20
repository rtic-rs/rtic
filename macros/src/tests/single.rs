use quote::quote;
use rtfm_syntax::Settings;

#[test]
fn analyze() {
    let mut settings = Settings::default();
    settings.parse_extern_interrupt = true;
    let (app, analysis) = rtfm_syntax::parse2(
        quote!(device = pac),
        quote!(
            const APP: () = {
                #[task(priority = 1)]
                fn a(_: a::Context) {}

                #[task(priority = 2)]
                fn b(_: b::Context) {}

                // first interrupt is assigned to the highest priority dispatcher
                extern "C" {
                    fn B();
                    fn A();
                }
            };
        ),
        settings,
    )
    .unwrap();

    let analysis = crate::analyze::app(analysis, &app);
    let interrupts = &analysis.interrupts[&0];
    assert_eq!(interrupts.len(), 2);
    assert_eq!(interrupts[&2].to_string(), "B");
    assert_eq!(interrupts[&1].to_string(), "A");
}
