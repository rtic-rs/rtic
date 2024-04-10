# `stm32g030f6_periodic_prints`

An RTIC periodic print example intended for the stm32g030f6 chip.

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


## Run

The stm32g030f6 chip needs to be connected to the computer via an SWD probe, like a [J-Link EDU Mini].

Then, run:

```
cargo run --release
```

[J-Link EDU Mini]: https://www.segger.com/products/debug-probes/j-link/models/j-link-edu-mini/
