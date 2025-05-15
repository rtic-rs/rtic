//! Real-Time Interrupt-driven Concurrency (RTIC) framework for ARM Cortex-M microcontrollers.
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
//! This crate is compiled and tested with the latest stable toolchain (rolling).
//! If you run into compilation errors, try the latest stable release of the rust toolchain.
//!
//! # Semantic Versioning
//!
//! Like the Rust project, this crate adheres to [SemVer]: breaking changes in the API and semantics
//! require a *semver bump* (since 1.0.0 a new major version release), with the exception of breaking changes
//! that fix soundness issues -- those are considered bug fixes and can be landed in a new patch
//! release.
//!
//! [SemVer]: https://semver.org/spec/v2.0.0.html

#![deny(missing_docs)]
#![deny(rust_2021_compatibility)]
#![deny(rust_2018_compatibility)]
#![deny(rust_2018_idioms)]
#![no_std]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/rtic-rs/rtic/master/book/en/src/RTIC.svg",
    html_favicon_url = "https://raw.githubusercontent.com/rtic-rs/rtic/master/book/en/src/RTIC.svg"
)]
#![allow(clippy::inline_always)]
#![allow(unexpected_cfgs)]

pub use rtic_core::{Exclusive, Mutex, prelude as mutex_prelude};
pub use rtic_macros::app;

/// module `mutex::prelude` provides `Mutex` and multi-lock variants. Recommended over `mutex_prelude`
pub mod mutex {
    pub use rtic_core::Mutex;
    pub use rtic_core::prelude;
}

#[doc(hidden)]
pub mod export;

pub use export::pend;

use core::cell::UnsafeCell;

/// Internal replacement for `static mut T`
///
/// Used to represent RTIC Resources
///
/// Soundness:
/// 1) Unsafe API for internal use only
/// 2) ``get_mut(&self) -> *mut T``
///    returns a raw mutable pointer to the inner T
///    casting to &mut T is under control of RTIC
///    RTIC ensures &mut T to be unique under Rust aliasing rules.
///
///    Implementation uses the underlying ``UnsafeCell<T>``
///    self.0.get() -> *mut T
///
/// 3) get(&self) -> *const T
///    returns a raw immutable (const) pointer to the inner T
///    casting to &T is under control of RTIC
///    RTIC ensures &T to be shared under Rust aliasing rules.
///
///    Implementation uses the underlying ``UnsafeCell<T>``
///    self.0.get() -> *mut T, demoted to *const T
///
#[repr(transparent)]
pub struct RacyCell<T>(UnsafeCell<T>);

impl<T> RacyCell<T> {
    /// Create a ``RacyCell``
    #[inline(always)]
    pub const fn new(value: T) -> Self {
        RacyCell(UnsafeCell::new(value))
    }

    /// Get `*mut T`
    ///
    /// # Safety
    ///
    /// See documentation notes for [`RacyCell`]
    #[inline(always)]
    pub unsafe fn get_mut(&self) -> *mut T {
        self.0.get()
    }

    /// Get `*const T`
    ///
    /// # Safety
    ///
    /// See documentation notes for [`RacyCell`]
    #[inline(always)]
    pub unsafe fn get(&self) -> *const T {
        self.0.get()
    }
}

unsafe impl<T> Sync for RacyCell<T> {}
