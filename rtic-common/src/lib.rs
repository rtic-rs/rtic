//! Utility structs that can be useful to other subcrates.

#![cfg_attr(not(feature = "loom"), no_std)]
#![deny(missing_docs)]

#[cfg(test)]
#[macro_use]
extern crate std;

pub mod dropper;
pub mod wait_queue;
pub mod waker_registration;

#[cfg(feature = "loom")]
#[allow(missing_docs)]
pub mod loom_cs;
