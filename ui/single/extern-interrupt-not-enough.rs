#![no_main]

#[rtic::app(device = lm3s6965)]
const APP: () = {
    #[task]
    fn a(_: a::Context) {}
};
