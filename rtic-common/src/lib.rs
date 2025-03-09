//! Utility structs that can be useful to other subcrates.

#![cfg_attr(not(loom), no_std)]
#![deny(missing_docs)]

#[cfg(test)]
#[macro_use]
extern crate std;

pub mod dropper;
pub mod wait_queue;
pub mod waker_registration;

#[cfg(loom)]
#[allow(missing_docs)]
pub mod loom_cs;

#[cfg(not(loom))]
pub mod unsafecell;
