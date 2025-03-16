//! Synchronization primitives for asynchronous contexts.

#![cfg_attr(not(loom), no_std)]
#![deny(missing_docs)]

#[cfg(feature = "defmt-03")]
use defmt_03 as defmt;

pub mod arbiter;
pub mod channel;
pub use portable_atomic;
pub mod signal;

mod unsafecell;

#[cfg(test)]
#[macro_use]
extern crate std;

#[cfg(loom)]
mod loom_cs;
