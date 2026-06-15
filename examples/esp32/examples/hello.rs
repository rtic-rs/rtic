#![no_std]
#![no_main]

esp_bootloader_esp_idf::esp_app_desc!();

use esp_backtrace as _;
use esp_println::println;
use xtensa_lx_rt::entry;

#[entry]
fn main() -> ! {
    let _ = esp_hal::init(esp_hal::Config::default());
    println!("hello");
    loop {}
}
