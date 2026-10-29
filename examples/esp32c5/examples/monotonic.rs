#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![no_main]
#![no_std]
use esp_backtrace as _;

esp_bootloader_esp_idf::esp_app_desc!();

#[rtic::app(device = esp_hal, dispatchers = [])]
mod app {
    use esp_hal::Config;
    use esp_println::println;
    use rtic_monotonics::esp32c5::prelude::*;

    esp32c5_systimer_monotonic!(Mono);

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_cx: init::Context) -> (Shared, Local) {
        println!("init");
        let peripherals = esp_hal::init(Config::default());
        Mono::start(peripherals.SYSTIMER);

        foo::spawn().unwrap();
        bar::spawn().unwrap();
        baz::spawn().unwrap();

        (Shared {}, Local {})
    }

    #[task]
    async fn foo(_cx: foo::Context) {
        println!("hello from foo");
        Mono::delay(2_u64.secs()).await;
        println!("bye from foo");
    }

    #[task]
    async fn bar(_cx: bar::Context) {
        println!("hello from bar");
        Mono::delay(3_u64.secs()).await;
        println!("bye from bar");
    }

    #[task]
    async fn baz(_cx: baz::Context) {
        println!("hello from baz");
        Mono::delay(4_u64.secs()).await;
        println!("bye from baz");
    }
}
