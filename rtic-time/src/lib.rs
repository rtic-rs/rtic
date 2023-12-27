//! Time-related traits & structs.
//!
//! This crate contains basic definitions and utilities that can be used
//! to keep track of time.

#![no_std]
#![deny(missing_docs)]
#![allow(async_fn_in_trait)]

pub mod half_period_counter;
mod linked_list;
pub mod monotonic;
pub mod timer_queue;

/// This indicates that there was a timeout.
pub struct TimeoutError;
