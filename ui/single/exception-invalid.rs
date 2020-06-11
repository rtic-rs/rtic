#![no_main]

#[rtic::app(device = lm3s6965)]
const APP: () = {
    #[task(binds = NonMaskableInt)]
    fn nmi(_: nmi::Context) {}
};
