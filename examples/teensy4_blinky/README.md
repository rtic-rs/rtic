# Teensy4 RTIC Blink example

Working example of simple LED blinking application for Teensy4. Example uses monotonics API and peripherials access.

## How-to

### Prerequisites

The following hardware is required for the examples:
- A [Teensy 4.0](https://www.pjrc.com/store/teensy40.html)/[Teensy 4.1](https://www.pjrc.com/store/teensy41.html)/[Teensy MicroMod](https://www.sparkfun.com/products/16402) development board

The following software tools have to be installed:
- Python3 (as `python3`, or modify `run.py` to use the `python` binary)
- [`cargo-binutils`](https://crates.io/crates/cargo-binutils)
- [`teensy_loader_cli`](https://www.pjrc.com/teensy/loader_cli.html)


### Run

- Connect the Teensy to PC via USB cable.
- Press the `Reset`/`Boot` button on the Teensy.
- Run:
  ```bash
  cargo run --release
  ```
