#![no_main]

#[rtic_macros::mock_app(device = mock)]
mod app {
    #[task(binds = SysTick, only_same_priority_spawn_please_fix_me)]
    fn foo(_: foo::Context) {}
}
