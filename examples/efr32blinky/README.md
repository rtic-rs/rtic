# EFR32 RTIC blinky

RTIC blinky for Silicon Labs EFR32, demonstrating the `silabs` monotonics.
Two binaries:

- `blinky` — **TIMER0** monotonic (high-frequency, 1 MHz tick)
- `blinky_letimer` — **LETIMER** monotonic (32.768 kHz, runs in EM2+ deep sleep)

## Boards

Select exactly one board feature; the LED pin, low-frequency clock source and
memory map are chosen accordingly.

| feature | board | chip |
|---------|-------|------|
| `mg26` (default) | SiLabs brd2713a | EFR32MG26B420F3200IM68 |
| `mg24` | Seeed XIAO MG24 | EFR32MG24B220F1536IM48 |

## Build

```sh
cargo build                                        # mg26 (default)
cargo build --no-default-features --features mg24  # mg24
```

## Run (debug probe attached)

The chip comes from `PROBE_RS_CHIP` (defaulted to mg26 in `.cargo/config.toml`):

```sh
# mg26 (brd2713a)
cargo run --bin blinky

# mg24 (XIAO MG24) — override the chip
PROBE_RS_CHIP=EFR32MG24B220F1536IM48 cargo run --no-default-features --features mg24 --bin blinky
```

Swap `--bin blinky` for `--bin blinky_letimer` to drive the LETIMER monotonic.
