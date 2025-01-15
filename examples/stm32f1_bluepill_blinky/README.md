# STM32F103 Bluepill RTIC Blink example

Working example of simple LED blinking application for popular Bluepill boards based on the STM32F103C8 chip. Example uses schedule API and peripherials access. You will need `stlink v2` tool or other programmer to flash the board.

## How-to

### Terminal workflow

Rust embedded relies heavily on `terminal workflow`, you will enter commands in the terminal. This can be strange at first, but this enables usage of great things like continious integration tools.

For Mac OS X consider using `iTerm2` instead of Terminal application.
For Windows consider using `powershell` (win + r -> powershell -> enter -> cd examples\stm3f1_bluepill_blinky)

### Build

Run `cargo build` to compile the code. If you run it for the first time, it will take some time to download and compile dependencies. After that, you will see comething like:

```shell
$ cargo build
Finished dev [optimized + debuginfo] target(s) in 0.10s
```

If you see warnings, feel free to ask for help in chat or issues of this repo.

### Connect the board

You need to connect you bluepill board to ST-Link and connect pins:

| BOARD |    | ST-LINK |
|-------|----|---------|
| GND   | -> | GND     |
| 3.3V  | -> | 3.3V    |
| SWCLK | -> | SWCLK   |
| SWDIO | -> | SWDIO   |

Plug in ST-Link to USB port and wait it to initialize.

### Flashing and running

Install `cargo embed` from probe-rs tools by following the instructions at https://probe.rs/docs/getting-started/installation/.
Flashing with a standard STLink v2 is easy with `cargo-embed`:

```shell
$ cargo embed --release
```
