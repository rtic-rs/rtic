[![crates.io](https://img.shields.io/crates/v/rtic-monotonics.svg)](https://crates.io/crates/rtic-monotonics)
[![crates.io](https://img.shields.io/crates/d/rtic-monotonics.svg)](https://crates.io/crates/rtic-monotonics)

# `rtic-monotonics`

> Reference implementations of the Real-Time Interrupt-driven Concurrency (RTIC) Monotonics timers

Uses [`rtic-time`](https://github.com/rtic-rs/rtic/tree/master/rtic-time) defined [`Monotonic`](https://docs.rs/rtic-time/latest/rtic_time/timer_queue/trait.Monotonic.html) trait.

`rtic-monotonics` is for RTIC v2.

For RTIC v1 see [`rtic-monotonic`](https://github.com/rtic-rs/rtic-monotonic)

## [Documentation](https://docs.rs/rtic-monotonics)

[RTIC book: chapter on monotonics](https://rtic.rs/2/book/en/by-example/delay.html)

### [Changelog `rtic-monotonics`](https://github.com/rtic-rs/rtic/blob/master/rtic-monotonics/CHANGELOG.md)

## Usage

Enable the feature for the timer you want to use. For chips accessed through a
metapac (STM32, Silabs), also enable the chip feature on your own metapac
dependency.

### SysTick (any Cortex-M)

```toml
rtic-monotonics = { version = "3", features = ["cortex-m-systick"] }
```

```rust
use rtic_monotonics::systick::prelude::*;

systick_monotonic!(Mono, 1_000);

fn init() {
    let core_peripherals = cortex_m::Peripherals::take().unwrap();
    Mono::start(core_peripherals.SYST, 12_000_000);
}
```

### STM32 timer

```toml
rtic-monotonics = { version = "3", features = ["stm32_tim2"] }
stm32-metapac = { version = "21", features = ["stm32f411ce"] }
```

```rust
use rtic_monotonics::stm32::prelude::*;

stm32_tim2_monotonic!(Mono, 1_000_000);

fn init() {
    // TIM2 peripheral clock frequency.
    Mono::start(48_000_000);
}
```

### Silabs timer

```toml
rtic-monotonics = { version = "3", features = ["silabs_timer0"] }
silabs-metapac = { version = "0.4", features = ["efr32mg24b220f1536im48"] }
```

```rust
use rtic_monotonics::silabs::timer::prelude::*;

silabs_timer0_monotonic!(Mono, 1_000_000);

fn init() {
    // TIMER0 peripheral clock frequency.
    Mono::start(39_000_000);
}
```

Once started, the monotonic is used the same way for every backend:

```rust
async fn usage() {
    let timestamp = Mono::now();
    Mono::delay(100.millis()).await;
}
```

## Supported Platforms

The following microcontroller families feature efficient monotonics using peripherals.
Refer to the [crate documentation](https://docs.rs/rtic-monotonics) for more details.

- Any Cortex-M (SysTick)
- STM32
- RP2040 / RP235x
- i.MX RT
- nRF
- ESP32-C3 / ESP32-C6
- Silabs EFR32
- ATSAMD (via the [`atsamd-hal`](https://docs.rs/atsamd-hal) crate)

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
