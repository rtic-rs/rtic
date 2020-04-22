#![no_main]

#[rtic::app(device = lm3s6965)]
mod APP {
    #[task]
    fn a(_: a::Context) {}
}
