//! Crate

#![no_std]
#![no_main]
#![deny(missing_docs)]
//deny_warnings_placeholder_for_ci
#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]

pub use rtic_time::{Monotonic, TimeoutError, TimerQueue};

#[cfg(feature = "cortex-m-systick")]
pub mod systick;

#[cfg(feature = "rp2040")]
pub mod rp2040;
