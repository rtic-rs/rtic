# STM32F411CEU6 RTC interrup example 

Working example to configure the internal RTC of the STM32F411CEU6 present on the Blackpill board.  
After configured, it will listen to periodic wake-up interrupts happening every 10 seconds until the button on GPIO PA0 is pressed. 

## How-to

### Build
Run `cargo build --release` to compile the code. If you run it for the first time, it will take some time to download and compile dependencies.

### Run
Install `probe-rs` and configure it using the [debugging extension for VScode](https://probe.rs/docs/tools/debugger/).
