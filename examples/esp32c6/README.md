### ESP32-C3 RTIC template
This crate showcases a simple RTIC application for the ESP32-C6.

## Prerequisites

# Nightly Rust
The ESP32-C6 HAL requires a nightly build of Rust.
Following the example of the (Espressif no_std book)[https://docs.esp-rs.org/no_std-training/02_2_software.html], we use this specific build:
```rustup toolchain install nightly-2023-11-14 --component rust-src --target riscv32imac-unknown-none-elf```

# Espressif toolchain

This crate uses the most convenient option in ``cargo-espflash`` and ``espflash``
```cargo install cargo-espflash espflash```

## Running the crate

```cargo run --example sw_and_hw --features=riscv-esp32c6-backend (--release)```

should do the trick.

# Expected behavior
The program
- Prints ``init``
- Enters a high prio task
- During the execution of the high prio task, the button should be non-functional
- Pends a low prio task
- Exits the high prio task
- Enters the low prio task
- During the execution of the low prio task, the button should be functional.
- Exits the low prio task
- Prints ``idle``

