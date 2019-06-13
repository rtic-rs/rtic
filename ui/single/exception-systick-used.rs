#![no_main]

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    #[exception]
    fn SysTick(_: SysTick::Context) {}

    #[task(schedule = [foo])]
    fn foo(_: foo::Context) {}
};
