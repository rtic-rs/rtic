//! Synchronization primitives for asynchronous contexts.

#![no_std]
#![deny(missing_docs)]
//deny_warnings_placeholder_for_ci

pub mod arbiter;
pub mod channel;

#[cfg(test)]
#[macro_use]
extern crate std;
