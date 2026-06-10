# EFR32 RTIC blinky

RTIC blinky for Silicon Labs EFR32, demonstrating the `silabs` monotonics.
Two binaries:

- `blinky` — **TIMER** monotonic (high-frequency, 1 MHz tick), instance chosen by `timerN`
- `blinky_letimer` — **LETIMER** monotonic (32.768 kHz, runs in EM2+ deep sleep)

## Features

Select exactly one **board** and (for `blinky`) one **timer**.

| board | board feature | chip |
|-------|---------------|------|
| UG613 MGM260P Module Explorer Kit | `mgm260p` (default) | EFR32MG26B420F3200IM68 |
| Seeed Studio XIAO MG24 | `xiao-mg24` | EFR32MG24B220F1536IM48 |

`timer0`..`timer9` pick which `TIMER` instance `blinky` uses (default `timer0`).
Both boards have `TIMER0..4`; the MGM260P additionally has `TIMER5..9`.

## Build

```sh
cargo build                                                   # mgm260p + timer0 (default)
cargo build --no-default-features --features xiao-mg24,timer0 # XIAO MG24
```

## Run (debug probe attached)

The chip comes from `PROBE_RS_CHIP` (defaulted to the MGM260P in `.cargo/config.toml`):

```sh
# MGM260P Explorer Kit, default timer0
cargo run --bin blinky

# XIAO MG24, on TIMER3 — override the chip + pick the timer
PROBE_RS_CHIP=EFR32MG24B220F1536IM48 cargo run --no-default-features --features xiao-mg24,timer3 --bin blinky
```

Swap `--bin blinky` for `--bin blinky_letimer` to drive the LETIMER monotonic.
