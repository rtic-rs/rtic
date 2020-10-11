#![no_main]

#[rtic::app(device = lm3s6965, monotonic = rtic::cyccnt::CYCCNT)]
mod app {
    #[task(binds = SysTick)]
    fn sys_tick(_: sys_tick::Context) {}
}
