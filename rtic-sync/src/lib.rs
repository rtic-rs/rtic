//! Synchronization primitives for asynchronous contexts.

#![no_std]
#![deny(missing_docs)]

pub mod arbiter;
pub mod channel;
pub use portable_atomic;

#[cfg(test)]
#[macro_use]
extern crate std;
