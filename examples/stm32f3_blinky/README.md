# STM32F3 RTIC Blink example

Working example of simple LED blinking application for STM32 F303 Nucleo-64 board based on the STM32F303RE chip. Example uses schedule API and peripherials access. This example is based on blue-pill blinky example.

## How-to

### Build

Run `cargo build --release` to compile the code. If you run it for the first time, it will take some time to download and compile dependencies.

After that, you can use for example the cargo-embed tool to flash and run it

```bash
$ cargo embed
```

### Setup environment, flash and run program

In the [Discovery Book](https://rust-embedded.github.io/discovery) you find all needed informations to setup the environment, flash the controler and run the program.
