#![no_main]

#[rtic::app(device = lm3s6965)]
mod APP {
    #[task(binds = NonMaskableInt)]
    fn nmi(_: nmi::Context) {}
}
