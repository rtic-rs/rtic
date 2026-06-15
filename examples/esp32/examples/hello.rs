#![no_std]
#![no_main]

esp_bootloader_esp_idf::esp_app_desc!();

use esp_backtrace as _;
use esp_println::println;

#[rtic::app(device = esp32, dispatchers = [FROM_CPU_INTR0])]
mod app {
    use esp_println::println;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        let _ = esp_hal::init(esp_hal::Config::default());
        hello::spawn().ok();
        println!("init");
        (Shared {}, Local {})
    }

    #[task(priority = 1)]
    async fn hello(_: hello::Context) {
        println!("hello"); //won't show until i unstub stuff
    }
}
