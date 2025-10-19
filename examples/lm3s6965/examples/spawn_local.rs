#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = lm3s6965, dispatchers = [SSI0])]
mod app {
    use cortex_m_semihosting::hprintln;
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_cx: init::Context) -> (Shared, Local) {
        task1::spawn().unwrap();
        //task2::spawn(Default::default()).ok(); <--- This is rejected since it is a local task
        (Shared {}, Local {})
    }

    #[task(priority = 1)]
    async fn task1(cx: task1::Context) {
        hprintln!("Hello from task1!");
        cx.local_spawner.task2(Default::default()).unwrap();
        cx.local_spawner.task3(Default::default()).unwrap();
    }

    #[task(priority = 1, local_task = true)]
    async fn task2(_cx: task2::Context, _nsns: NotSendNotSync) {
        hprintln!("Hello from task2!");
    }

    // The value of `local_task` may be omitted and is in that case interpreted as `true`
    #[task(priority = 1, local_task)]
    async fn task3(_cx: task3::Context, _nsns: NotSendNotSync) {
        hprintln!("Hello from task3!");
    }
}

#[derive(Default, Debug)]
struct NotSendNotSync(core::marker::PhantomData<*mut u8>);
