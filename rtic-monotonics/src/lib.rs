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

/// This marker is implemented on an interrupt token to enforce that the right tokens
/// are given to the correct monotonic implementation.
///
/// This trait is implemented by this crate and not intended for user implementation.
///
/// # Safety
///
/// This is only safely implemented by this crate.
pub unsafe trait InterruptToken<Periperhal> {}
