//! Synchronization primitives for asynchronous contexts.

#![no_std]
#![deny(missing_docs)]

#[cfg(feature = "defmt-03")]
use defmt_03 as defmt;

pub mod arbiter;
pub mod channel;
pub use portable_atomic;
// pub mod signal;

#[cfg(test)]
#[macro_use]
extern crate std;
