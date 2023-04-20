//! Synchronization primitives for asynchronous contexts.

#![no_std]
#![deny(missing_docs)]

pub mod arbiter;
pub mod channel;

#[cfg(test)]
#[macro_use]
extern crate std;
