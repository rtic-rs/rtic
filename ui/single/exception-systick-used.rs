#![no_main]

#[rtfm::app(device = lm3s6965)]
const APP: () = {
    #[task(binds = SysTick)]
    fn sys_tick(_: sys_tick::Context) {}

    #[task(schedule = [foo])]
    fn foo(_: foo::Context) {}
};
