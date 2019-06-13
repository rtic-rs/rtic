#![no_main]

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    #[task]
    fn a(_: a::Context) {}
};
