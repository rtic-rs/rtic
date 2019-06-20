use quote::quote;
use rtfm_syntax::Settings;

#[test]
fn analyze() {
    let mut settings = Settings::default();
    settings.parse_cores = true;
    settings.parse_extern_interrupt = true;

    let (app, analysis) = rtfm_syntax::parse2(
        quote!(device = pac, cores = 2),
        quote!(
            const APP: () = {
                #[task(core = 0, priority = 1)]
                fn a(_: a::Context) {}

                #[task(core = 0, priority = 2)]
                fn b(_: b::Context) {}

                #[task(core = 1, priority = 1)]
                fn c(_: c::Context) {}

                #[task(core = 1, priority = 2)]
                fn d(_: d::Context) {}

                // first interrupt is assigned to the highest priority dispatcher
                extern "C" {
                    #[core = 0]
                    fn B();

                    #[core = 0]
                    fn A();

                    #[core = 1]
                    fn A();

                    #[core = 1]
                    fn C();
                }
            };
        ),
        settings,
    )
    .unwrap();

    let analysis = crate::analyze::app(analysis, &app);

    // first core
    let interrupts0 = &analysis.interrupts[&0];
    assert_eq!(interrupts0.len(), 2);
    assert_eq!(interrupts0[&2].to_string(), "B");
    assert_eq!(interrupts0[&1].to_string(), "A");

    // second core
    let interrupts1 = &analysis.interrupts[&1];
    assert_eq!(interrupts1.len(), 2);
    assert_eq!(interrupts1[&2].to_string(), "A");
    assert_eq!(interrupts1[&1].to_string(), "C");
}
