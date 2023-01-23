//! Crate

#![no_std]
#![no_main]
#![deny(missing_docs)]
#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]

pub use rtic_timer::{Monotonic, TimeoutError, TimerQueue};

pub mod systick_monotonic;
