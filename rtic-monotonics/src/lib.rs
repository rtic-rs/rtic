//! Crate

#![no_std]
#![deny(missing_docs)]
//deny_warnings_placeholder_for_ci
#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]

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
