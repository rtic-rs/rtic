#![no_main]

#[rtic::app(device = lm3s6965)]
mod app {
    #[task]
    fn a(_: a::Context) {}
}
