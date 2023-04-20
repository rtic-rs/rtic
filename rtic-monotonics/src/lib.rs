//! In-tree implementations of the [`rtic_time::Monotonic`] (reexported) trait for
//! timers & clocks found on commonly used microcontrollers.
//!
//! To enable the implementations, you must enable a feature for the specific MCU you're targeting.
//!
//! # Cortex-M Systick
//! The [`systick`] monotonic works on all cortex-M parts, and requires that the feature `cortex-m-systick` is enabled.
//!
//! # RP2040
//! The RP2040 monotonics require that the `rp2040` feature is enabled.
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
// RUSTFLAGS="--cfg docsrs" cargo doc --featuers cortex-m-systick,rp2040,nrf52840

#![no_std]
#![deny(missing_docs)]
#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub use rtic_time::{Monotonic, TimeoutError, TimerQueue};

#[cfg(feature = "cortex-m-systick")]
pub mod systick;

#[cfg(feature = "rp2040")]
pub mod rp2040;

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

/// This marker is implemented on an interrupt token to enforce that the right tokens
/// are given to the correct monotonic implementation.
///
/// This trait is implemented by this crate and not intended for user implementation.
///
/// # Safety
///
/// This is only safely implemented by this crate.
pub unsafe trait InterruptToken<Periperhal> {}
