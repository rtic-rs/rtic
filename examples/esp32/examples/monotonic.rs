#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![no_main]
#![no_std]
use esp_backtrace as _;

esp_bootloader_esp_idf::esp_app_desc!();

#[rtic::app(device = esp32, dispatchers = [FROM_CPU_INTR0])]
mod app {
    use rtic_monotonics::esp32::prelude::*;
    esp32_timg0_monotonic!(Mono);
    use esp_println::println;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_cx: init::Context) -> (Shared, Local) {
        let peripherals = esp_hal::init(esp_hal::Config::default());

        Mono::start(peripherals.TIMG0);

        foo::spawn().unwrap();
        bar::spawn().unwrap();
        baz::spawn().unwrap();

        println!("init");
        (Shared {}, Local {})
    }

    #[task(priority = 1)]
    async fn foo(_cx: foo::Context) {
        println!("hello from foo");
        Mono::delay(2_u64.secs()).await;
        println!("bye from foo");
    }

    #[task(priority = 1)]
    async fn bar(_cx: bar::Context) {
        println!("hello from bar");
        Mono::delay(3_u64.secs()).await;
        println!("bye from bar");
    }

    #[task(priority = 1)]
    async fn baz(_cx: baz::Context) {
        println!("hello from baz");
        Mono::delay(4_u64.secs()).await;
        println!("bye from baz");
    }
}
