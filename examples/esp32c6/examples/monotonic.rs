#![no_main]
#![no_std]
use esp_backtrace as _;
#[rtic::app(device = esp32c6, dispatchers = [])]
mod app {
    use rtic_monotonics::esp32c6::prelude::*;
    esp32c6_systimer_monotonic!(Mono);
    use esp_hal as _;
    use esp_println::println;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        println!("init");
        let timer = cx.device.SYSTIMER;

        Mono::start(timer);

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
