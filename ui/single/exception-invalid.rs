#![no_main]

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    #[task(binds = NonMaskableInt)]
    fn nmi(_: nmi::Context) {}
};
