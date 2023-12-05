# `nrf52840_blinky`

An RTIC blinky example intended for [nrf52840-dongle].

[nrf52840-dongle]: https://www.nordicsemi.com/Products/Development-hardware/nrf52840-dongle

## Dependencies

#### 1. `flip-link`:

```console
$ cargo install flip-link
```

#### 2. `probe-rs`:

``` console
$ # make sure to install v0.2.0 or later
$ cargo install probe-rs --features cli
```

#### 3. [`cargo-generate`]:

``` console
$ cargo install cargo-generate
```

[`cargo-generate`]: https://crates.io/crates/cargo-generate

> *Note:* You can also just clone this repository instead of using `cargo-generate`, but this involves additional manual adjustments.



## Run

The [nrf52840-dongle] needs to be connected to the computer via an SWD probe, like a [J-Link EDU Mini].

Then, run:

```
cargo run --release --bin blinky_timer
```

[J-Link EDU Mini]: https://www.segger.com/products/debug-probes/j-link/models/j-link-edu-mini/
