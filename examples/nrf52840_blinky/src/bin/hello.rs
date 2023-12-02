#![no_main]
#![no_std]

use nrf52840_blinky as _; // global logger + panicking-behavior + memory layout

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("Hello, world!");

    nrf52840_blinky::exit()
}
