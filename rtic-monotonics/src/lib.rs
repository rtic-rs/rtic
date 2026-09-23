//! In-tree implementations of the [`rtic_time::Timebase`] and [`rtic_time::Monotonic`]
//! (reexported) traits for timers & clocks found on commonly used microcontrollers.
//!
//! If you are using a microcontroller where CAS operations are not available natively, you might
//! have to enable the `critical-section` or `unsafe-assume-single-core` feature of the
//! [`portable-atomic`](https://docs.rs/portable-atomic/latest/portable_atomic/) dependency
//! yourself for this dependency to compile.
//!
//! To enable the implementations, you must enable a feature for the specific MCU you're targeting.
//!
//! # Cortex-M Systick
//! The `systick` monotonic works on all Arm Cortex-M parts, and requires that the feature `cortex-m-systick` is enabled.
//!
//! # RP2040
//! The RP2040 monotonics require that the `rp2040` feature is enabled.
//!
//! # RP2350
//! The RP2350 monotonics require that the `rp235x` feature is enabled.
//!
//! # i.MX RT
//! The i.MX RT monotonics require that the feature `imxrt_gpt1` or `imxrt_gpt2` is enabled.
//!
//! # nRF
//! nRF monotonics require that one of the available `nrf52*` features is enabled. Monotonic
//! implementations are available for both high-resolution TIMER and low-resolution RTC peripherals.
//!
//! All implementations of timers for the nRF52 family are documented here. Monotonics that
//! are not available on all parts in this family will have an `Available on crate features X only`
//! tag, describing what parts _do_ support that monotonic. Monotonics without an
//! `Available on crate features X only` tag are available on any `nrf52*` feature.
//!
//! # Silicon Labs (EFM, EFR)
//! Enable a peripheral selector feature (e.g. `silabs_letimer0`, `silabs_rtcc`, `silabs_timer0`)
//! plus the chip feature on your own `silabs-metapac` dependency
//! (e.g. `silabs-metapac/efr32mg22c224f512im40`).
//!
//! # ESP32C3 and ESP32C6
//! Enable either the `esp32c3-systimer` or `esp32c6-systimer` feature, as appropriate.
//!
//! # STM32
//! Enable one of the `stm32_tim*` features, plus the chip feature on your own `stm32-metapac`
//! dependency (e.g. `stm32-metapac/stm32g081kb`). Implementations are available for a
//! selection of STM32 timers.
//!
//! # ATSAMD
//! Monotonics for the ATSAMD family of parts using the real time clock (RTC) are provided in the
//! [`atsamd-hal`](https://docs.rs/atsamd-hal/latest/atsamd_hal/rtc/rtic/index.html)
//! crate with the `rtic` feature enabled.
//! # Priority of interrupt handlers
//!
//! The priority of the timer interrupt is given to `start` as a [`Priority`]. A task running at
//! or above it must not use blocking delays, as time stops advancing while the interrupt cannot
//! run.
//!
//! [`Priority::rtic_default`] is the priority RTIC reserves for async HAL drivers. It is 1 less
//! than the lowest hardware task priority, but never lower than 1 above the highest software task
//! priority, capped to the maximum priority. If there are no hardware tasks, it is the maximum
//! priority in the system.

// To build these docs correctly:
// RUSTFLAGS="--cfg docsrs" cargo +nightly doc --features thumbv7-backend,cortex-m-systick,rp2040,nrf52840,imxrt_gpt1,imxrt_gpt2,imxrt-ral/imxrt1011,stm32-metapac/stm32h725ag,stm32_tim2,stm32_tim3,stm32_tim4,stm32_tim5,stm32_tim15

#![no_std]
#![deny(missing_docs)]
#![allow(incomplete_features)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub use fugit;
pub use rtic_time::{
    self, monotonic::TimerQueueBasedMonotonic, timer_queue::TimerQueueBackend, Monotonic, Timebase,
    TimeoutError,
};

#[cfg(feature = "esp32c3-systimer")]
pub mod esp32c3;

#[cfg(feature = "esp32c6-systimer")]
pub mod esp32c6;

#[cfg(feature = "cortex-m-systick")]
pub mod systick;

#[cfg(feature = "rp2040")]
pub mod rp2040;

#[cfg(feature = "rp235x")]
pub mod rp235x;

#[cfg(feature = "imxrt")]
pub mod imxrt;

#[cfg(any(
    feature = "nrf52805",
    feature = "nrf52810",
    feature = "nrf52811",
    feature = "nrf52832",
    feature = "nrf52833",
    feature = "nrf52840",
    feature = "nrf5340-app",
    feature = "nrf5340-net",
    feature = "nrf9160-ns",
    feature = "nrf9160-s",
    feature = "nrf9151-ns",
    feature = "nrf9151-s",
    feature = "nrf9161-ns",
    feature = "nrf9161-s",
))]
pub mod nrf;

#[cfg(feature = "stm32-metapac")]
pub mod stm32;

#[cfg(feature = "silabs")]
pub mod silabs;

/// Logical priority of a monotonic's timer interrupt, where 1 is the lowest.
///
/// `PRIO_BITS` is the number of priority bits of the interrupt controller. Each monotonic module
/// has a `Priority` alias for its chip.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Priority<const PRIO_BITS: u8>(core::num::NonZeroU8);

impl<const PRIO_BITS: u8> Priority<PRIO_BITS> {
    /// Creates a priority.
    ///
    /// Panics if `prio` is 0 or above the highest priority of the interrupt controller.
    pub const fn new(prio: u8) -> Self {
        assert!(
            prio as u16 <= 1 << PRIO_BITS,
            "interrupt priority is above the highest priority"
        );

        match core::num::NonZeroU8::new(prio) {
            Some(prio) => Self(prio),
            None => panic!("interrupt priority must be at least 1"),
        }
    }

    /// The priority RTIC reserves for async HAL drivers, see the
    /// [crate docs](crate#priority-of-interrupt-handlers).
    ///
    /// Only available in applications using `#[rtic::app]`, which generates the value.
    #[inline]
    pub fn rtic_default() -> Self {
        extern "C" {
            static RTIC_ASYNC_MAX_LOGICAL_PRIO: u8;
        }

        // SAFETY: `#[rtic::app]` defines this as an immutable `u8`.
        Self::new(unsafe { RTIC_ASYNC_MAX_LOGICAL_PRIO }.max(1))
    }

    /// The logical priority.
    pub const fn get(self) -> u8 {
        self.0.get()
    }
}

#[allow(dead_code)]
pub(crate) const fn cortex_logical2hw(logical: u8, nvic_prio_bits: u8) -> u8 {
    ((1 << nvic_prio_bits) - logical) << (8 - nvic_prio_bits)
}

#[cfg(any(
    feature = "silabs",
    feature = "rp235x",
    feature = "rp2040",
    feature = "nrf52805",
    feature = "nrf52810",
    feature = "nrf52811",
    feature = "nrf52832",
    feature = "nrf52833",
    feature = "nrf52840",
    feature = "nrf5340-app",
    feature = "nrf5340-net",
    feature = "nrf9160-ns",
    feature = "nrf9160-s",
    feature = "nrf9151-ns",
    feature = "nrf9151-s",
    feature = "nrf9161-ns",
    feature = "nrf9161-s",
    feature = "imxrt",
    feature = "stm32-metapac",
))]
pub(crate) unsafe fn set_monotonic_prio<const PRIO_BITS: u8>(
    interrupt: impl cortex_m::interrupt::InterruptNumber,
    prio: Priority<PRIO_BITS>,
) {
    let hw_prio = crate::cortex_logical2hw(prio.get(), PRIO_BITS);

    // We take ownership of the entire IRQ and all settings to it, we only change settings
    // for the IRQ we control.
    // This will also compile-error in case the NVIC changes in size.
    let mut nvic: cortex_m::peripheral::NVIC = core::mem::transmute(());

    nvic.set_priority(interrupt, hw_prio);
}
