//! Real-Time Interrupt-driven Concurrency (RTIC) framework for ARM Cortex-M microcontrollers
//!
//! **HEADS UP** This is an **beta** pre-release; there may be breaking changes in the API and
//! semantics before a proper release is made.
//!
//! **IMPORTANT**: This crate is published as [`cortex-m-rtic`] on crates.io but the name of the
//! library is `rtic`.
//!
//! [`cortex-m-rtic`]: https://crates.io/crates/cortex-m-rtic
//!
//! The user level documentation can be found [here].
//!
//! [here]: https://rtic.rs
//!
//! Don't forget to check the documentation of the `#[app]` attribute (listed under the reexports
//! section), which is the main component of the framework.
//!
//! # Minimum Supported Rust Version (MSRV)
//!
//! This crate is guaranteed to compile on stable Rust 1.36 (2018 edition) and up. It *might*
//! compile on older versions but that may change in any new patch release.
//!
//! # Semantic Versioning
//!
//! Like the Rust project, this crate adheres to [SemVer]: breaking changes in the API and semantics
//! require a *semver bump* (a new minor version release), with the exception of breaking changes
//! that fix soundness issues -- those are considered bug fixes and can be landed in a new patch
//! release.
//!
//! [SemVer]: https://semver.org/spec/v2.0.0.html

#![deny(missing_docs)]
#![deny(rust_2018_compatibility)]
#![deny(rust_2018_idioms)]
#![deny(warnings)]
#![no_std]

use cortex_m::{interrupt::InterruptNumber, peripheral::NVIC};
pub use cortex_m_rtic_macros::app;
pub use rtic_core::{prelude as mutex_prelude, Exclusive, Mutex};
pub use rtic_monotonic::{self, embedded_time as time, Monotonic};

#[doc(hidden)]
pub mod export;
#[doc(hidden)]
mod tq;

/// Sets the given `interrupt` as pending
///
/// This is a convenience function around
/// [`NVIC::pend`](../cortex_m/peripheral/struct.NVIC.html#method.pend)
pub fn pend<I>(interrupt: I)
where
    I: InterruptNumber,
{
    NVIC::pend(interrupt)
}

use core::cell::UnsafeCell;

/// Internal replacement for `static mut T`
#[repr(transparent)]
pub struct RacyCell<T>(UnsafeCell<T>);

impl<T> RacyCell<T> {
    /// Create a RacyCell
    #[inline(always)]
    pub const fn new(value: T) -> Self {
        RacyCell(UnsafeCell::new(value))
    }

    /// Get `&mut T`
    #[inline(always)]
    pub unsafe fn get_mut_unchecked(&self) -> &mut T {
        &mut *self.0.get()
    }

    /// Get `&T`
    #[inline(always)]
    pub unsafe fn get_unchecked(&self) -> &T {
        &*self.0.get()
    }
}

unsafe impl<T> Sync for RacyCell<T> {}
