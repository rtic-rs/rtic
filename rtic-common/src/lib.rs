//! Utility structs that can be useful to other subcrates.

#![no_std]
#![deny(missing_docs)]
//deny_warnings_placeholder_for_ci

#[cfg(test)]
#[macro_use]
extern crate std;

pub mod dropper;
pub mod wait_queue;
pub mod waker_registration;
