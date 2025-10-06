#![no_main]
#![no_std]

use core::marker::PhantomData;
use rtic::app;
use {defmt_rtt as _, panic_probe as _};
pub mod pac {
    pub use embassy_stm32::pac::Interrupt as interrupt;
    pub use embassy_stm32::pac::*;
}

#[app(device = pac, peripherals = false, dispatchers = [SPI1])]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_cx: init::Context) -> (Shared, Local) {
        task1::spawn().ok();
        //task2::spawn(Default::default()).ok(); <--- This is rejected since it is a local task
        (Shared {}, Local {})
    }

    #[task(priority = 1)]
    async fn task1(cx: task1::Context) {
        defmt::info!("Hello from task1!");
        cx.local_spawner.task2(Default::default()).ok();
    }

    #[task(priority = 1, is_local_task = true)]
    async fn task2(_cx: task2::Context, _nsns: super::NotSendNotSync) {
        defmt::info!("Hello from task1!");
    }
}

#[derive(Default)]
struct NotSendNotSync(PhantomData<*mut u8>);
