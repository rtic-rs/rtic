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
//!
//! # Cargo features
//!
//! - `heterogeneous`. This opt-in feature enables the *experimental* heterogeneous multi-core
//! support. This feature depends on unstable feature and requires the use of the nightly channel.
//!
//! - `homogeneous`. This opt-in feature enables the *experimental* homogeneous multi-core support.

#![deny(missing_docs)]
#![deny(rust_2018_compatibility)]
#![deny(rust_2018_idioms)]
#![deny(warnings)]
#![no_std]

use cortex_m::{
    interrupt::Nr,
    peripheral::{CBP, CPUID, DCB, DWT, FPB, FPU, ITM, MPU, NVIC, SCB, TPIU},
};
#[cfg(all(not(feature = "heterogeneous"), not(feature = "homogeneous")))]
use cortex_m_rt as _; // vector table
pub use cortex_m_rtic_macros::app;
pub use embedded_time as time;
pub use rtic_core::{Exclusive, Mutex};

#[doc(hidden)]
pub mod export;
#[doc(hidden)]
mod tq;

/// `cortex_m::Peripherals` minus `SYST`
#[allow(non_snake_case)]
pub struct Peripherals {
    /// Cache and branch predictor maintenance operations (not present on Cortex-M0 variants)
    pub CBP: CBP,

    /// CPUID
    pub CPUID: CPUID,

    /// Debug Control Block
    pub DCB: DCB,

    /// Data Watchpoint and Trace unit
    pub DWT: DWT,

    /// Flash Patch and Breakpoint unit (not present on Cortex-M0 variants)
    pub FPB: FPB,

    /// Floating Point Unit (only present on `thumbv7em-none-eabihf`)
    pub FPU: FPU,

    /// Instrumentation Trace Macrocell (not present on Cortex-M0 variants)
    pub ITM: ITM,

    /// Memory Protection Unit
    pub MPU: MPU,

    /// Nested Vector Interrupt Controller
    pub NVIC: NVIC,

    /// System Control Block
    pub SCB: SCB,

    // SysTick: System Timer
    // pub SYST: SYST,
    /// Trace Port Interface Unit (not present on Cortex-M0 variants)
    pub TPIU: TPIU,
}

impl From<cortex_m::Peripherals> for Peripherals {
    fn from(p: cortex_m::Peripherals) -> Self {
        Self {
            CBP: p.CBP,
            CPUID: p.CPUID,
            DCB: p.DCB,
            DWT: p.DWT,
            FPB: p.FPB,
            FPU: p.FPU,
            ITM: p.ITM,
            MPU: p.MPU,
            NVIC: p.NVIC,
            SCB: p.SCB,
            TPIU: p.TPIU,
        }
    }
}

/// A monotonic clock / counter
pub trait Monotonic: time::Clock {
    /// Resets the counter to *zero*
    ///
    /// # Safety
    ///
    /// This function will be called *exactly once* by the RTIC runtime after `#[init]` returns and
    /// before tasks can start; this is also the case in multi-core applications. User code must
    /// *never* call this function.
    unsafe fn reset();
}

/// A marker trait that indicates that it is correct to use this type in multi-core context
pub trait MultiCore {}

/// Sets the given `interrupt` as pending
///
/// This is a convenience function around
/// [`NVIC::pend`](../cortex_m/peripheral/struct.NVIC.html#method.pend)
pub fn pend<I>(interrupt: I)
where
    I: Nr,
{
    NVIC::pend(interrupt)
}
