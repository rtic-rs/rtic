#![no_main]

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    #[exception]
    fn NonMaskableInt(_: NonMaskableInt::Context) {}
};
