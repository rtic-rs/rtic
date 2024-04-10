//! In-tree implementations of the [`rtic_time::Monotonic`] (reexported) trait for
//! timers & clocks found on commonly used microcontrollers.
//!
//! To enable the implementations, you must enable a feature for the specific MCU you're targeting.
//!
//! # Cortex-M Systick
//! The `systick` monotonic works on all cortex-M parts, and requires that the feature `cortex-m-systick` is enabled.
//!
//! # RP2040
//! The RP2040 monotonics require that the `rp2040` feature is enabled.
//!
//! # i.MX RT
//! The i.MX RT monotonics require that the feature `imxrt_gpt1` or `imxrt_gpt2` is enabled.
//!
//! # nRF
//! nRF monotonics require that one of the available `nrf52*` features is enabled.
//!
//! All implementations of timers for the nRF52 family are documented here. Monotonics that
//! are not available on all parts in this family will have an `Available on crate features X only`
//! tag, describing what parts _do_ support that monotonic. Monotonics without an
//! `Available on crate features X only` tag are available on any `nrf52*` feature.
//!
// To build these docs correctly:
// RUSTFLAGS="--cfg docsrs" cargo +nightly doc --features thumbv7-backend,cortex-m-systick,rp2040,nrf52840,imxrt_gpt1,imxrt_gpt2,imxrt-ral/imxrt1011,stm32h725ag,stm32_tim2,stm32_tim3,stm32_tim4,stm32_tim5,stm32_tim15

#![no_std]
#![deny(missing_docs)]
#![allow(incomplete_features)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub use fugit;
pub use rtic_time::{
    self, monotonic::TimerQueueBasedMonotonic, timer_queue::TimerQueueBackend, Monotonic,
    TimeoutError,
};

#[cfg(feature = "cortex-m-systick")]
pub mod systick;

#[cfg(feature = "rp2040")]
pub mod rp2040;

#[cfg(feature = "imxrt")]
pub mod imxrt;

#[cfg(any(
    feature = "nrf52810",
    feature = "nrf52811",
    feature = "nrf52832",
    feature = "nrf52833",
    feature = "nrf52840",
    feature = "nrf5340-app",
    feature = "nrf5340-net",
    feature = "nrf9160",
))]
pub mod nrf;

// Notice that `stm32` is not a feature, it is a compilation flag set in build.rs.
#[cfg(stm32)]
pub mod stm32;

#[allow(dead_code)]
pub(crate) const fn cortex_logical2hw(logical: u8, nvic_prio_bits: u8) -> u8 {
    ((1 << nvic_prio_bits) - logical) << (8 - nvic_prio_bits)
}

#[cfg(any(
    feature = "rp2040",
    feature = "nrf52810",
    feature = "nrf52811",
    feature = "nrf52832",
    feature = "nrf52833",
    feature = "nrf52840",
    feature = "nrf5340-app",
    feature = "nrf5340-net",
    feature = "nrf9160",
    feature = "imxrt",
    stm32,
))]
pub(crate) unsafe fn set_monotonic_prio(
    prio_bits: u8,
    interrupt: impl cortex_m::interrupt::InterruptNumber,
) {
    extern "C" {
        static RTIC_ASYNC_MAX_LOGICAL_PRIO: u8;
    }

    let max_prio = RTIC_ASYNC_MAX_LOGICAL_PRIO.max(1).min(1 << prio_bits);

    let hw_prio = crate::cortex_logical2hw(max_prio, prio_bits);

    // We take ownership of the entire IRQ and all settings to it, we only change settings
    // for the IRQ we control.
    // This will also compile-error in case the NVIC changes in size.
    let mut nvic: cortex_m::peripheral::NVIC = core::mem::transmute(());

    nvic.set_priority(interrupt, hw_prio);
}
