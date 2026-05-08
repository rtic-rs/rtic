# STM32F411CEU6 PWM GENERATION + DWT profiler
Working example to generate a PWM signal on the STM32F411CEU6 present on a Blackpill board.  


This example implements:

- 1 kHz PWM output on TIM1 (PA9)
- Button-controlled duty cycle ramp (EXTI interrupt)
- LED heartbeat task (idle loop)
- Cycle-accurate performance measurement using DWT

---


## PWM Control (TIM1)
- PWM frequency: **1 kHz**
- Output pin: **PA9 (TIM1_CH2)**
- Ramps up/down on steps of 5% automatically on button press

---

## Button Interrupt (EXTI0 / PA0)
Each button press:
- Changes PWM duty cycle (5% steps)
- Reverses direction at 0% and 100%
- Logs updated duty cycle

---

## Performance Profiling (DWT)
The firmware uses the Cortex-M **DWT cycle counter** to measure:

- Time taken to update PWM duty cycle
- CPU cycles per update
- Conversion to microseconds (assuming 84 MHz system clock and 25MHZ HSE)

Example log:
```
[INFO ] BOOT: init start
[INFO ] DWT ready
[INFO ] PWM ready
[INFO ] IRQ ready
[INFO ] IDLE running
[INFO ] IRQ
[INFO ] PWM duty=55%, update=410 cycles (4 us)
[INFO ] IRQ
[INFO ] PWM duty=60%, update=410 cycles (4 us)
```

## How-to

### Build
Run `cargo build --release` to compile the code. If you run it for the first time, it will take some time to download and compile dependencies.

### Run
Install `probe-rs` and configure it using the [debugging extension for VScode](https://probe.rs/docs/tools/debugger/).  


