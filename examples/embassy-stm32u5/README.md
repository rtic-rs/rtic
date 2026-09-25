# STM32U5 RTIC Blink example

Working example of simple LED blinking application for the NUCLEO-U575ZI-Q development board using [embassy-stm32](https://crates.io/crates/embassy-stm32) HAL where the blinking frequency can be tuned using the user button. This example shows how to configure the system ticker and how to use a custom linker file.

## How-to

### Build

Run `cargo build --release` to compile the code and `cargo run --release` to flash it.
