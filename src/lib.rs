//! Real Time For the Masses (RTFM) framework for ARM Cortex-M microcontrollers
//!
//! **IMPORTANT**: This crate is published as [`cortex-m-rtfm`] on crates.io but the name of the
//! library is `rtfm`.
//!
//! [`cortex-m-rtfm`]: https://crates.io/crates/cortex-m-rtfm
//!
//! The user level documentation can be found [here].
//!
//! [here]: https://japaric.github.io/cortex-m-rtfm/book/
//!
//! Don't forget to check the documentation of the [`#[app]`] attribute, which is the main component
//! of the framework.
//!
//! [`#[app]`]: ../cortex_m_rtfm_macros/attr.app.html
//!
//! # Cargo features
//!
//! - `timer-queue`. This opt-in feature enables the `schedule` API which can be used to schedule
//! tasks to run in the future. Also see [`Instant`] and [`Duration`].
//!
//! [`Instant`]: struct.Instant.html
//! [`Duration`]: struct.Duration.html

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]

#[cfg(feature = "timer-queue")]
use core::cmp::Ordering;
use core::{fmt, ops};

#[cfg(not(feature = "timer-queue"))]
use cortex_m::peripheral::SYST;
use cortex_m::{
    interrupt::Nr,
    peripheral::{CBP, CPUID, DCB, DWT, FPB, FPU, ITM, MPU, NVIC, SCB, TPIU},
};
pub use cortex_m_rtfm_macros::app;

#[doc(hidden)]
pub mod export;
#[doc(hidden)]
#[cfg(feature = "timer-queue")]
mod tq;

#[cfg(all(feature = "timer-queue", armv6m))]
compile_error!(
    "The `timer-queue` feature is currently not supported on ARMv6-M (`thumbv6m-none-eabi`)"
);

/// Core peripherals
///
/// This is `cortex_m::Peripherals` minus the peripherals that the RTFM runtime uses
///
/// - The `NVIC` field is never present.
/// - When the `timer-queue` feature is enabled the following fields are *not* present: `DWT` and
/// `SYST`.
#[allow(non_snake_case)]
pub struct Peripherals<'a> {
    /// Cache and branch predictor maintenance operations (not present on Cortex-M0 variants)
    pub CBP: CBP,

    /// CPUID
    pub CPUID: CPUID,

    /// Debug Control Block (by value if the `timer-queue` feature is disabled)
    #[cfg(feature = "timer-queue")]
    pub DCB: &'a mut DCB,

    /// Debug Control Block (borrowed if the `timer-queue` feature is enabled)
    #[cfg(not(feature = "timer-queue"))]
    pub DCB: DCB,

    /// Data Watchpoint and Trace unit (not present if the `timer-queue` feature is enabled)
    #[cfg(not(feature = "timer-queue"))]
    pub DWT: DWT,

    /// Flash Patch and Breakpoint unit (not present on Cortex-M0 variants)
    pub FPB: FPB,

    /// Floating Point Unit (only present on `thumbv7em-none-eabihf`)
    pub FPU: FPU,

    /// Instrumentation Trace Macrocell (not present on Cortex-M0 variants)
    pub ITM: ITM,

    /// Memory Protection Unit
    pub MPU: MPU,

    // Nested Vector Interrupt Controller
    // pub NVIC: NVIC,
    /// System Control Block
    pub SCB: &'a mut SCB,

    /// SysTick: System Timer (not present if the `timer-queue` is enabled)
    #[cfg(not(feature = "timer-queue"))]
    pub SYST: SYST,

    /// Trace Port Interface Unit (not present on Cortex-M0 variants)
    pub TPIU: TPIU,
}

/// A measurement of a monotonically nondecreasing clock. Opaque and useful only with `Duration`
///
/// This data type is only available when the `timer-queue` feature is enabled
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg(feature = "timer-queue")]
pub struct Instant(i32);

#[cfg(feature = "timer-queue")]
impl Instant {
    /// IMPLEMENTATION DETAIL. DO NOT USE
    #[doc(hidden)]
    pub fn artificial(timestamp: i32) -> Self {
        Instant(timestamp)
    }

    /// Returns an instant corresponding to "now"
    pub fn now() -> Self {
        Instant(DWT::get_cycle_count() as i32)
    }

    /// Returns the amount of time elapsed since this instant was created.
    pub fn elapsed(&self) -> Duration {
        Instant::now() - *self
    }

    /// Returns the amount of time elapsed from another instant to this one.
    pub fn duration_since(&self, earlier: Instant) -> Duration {
        let diff = self.0 - earlier.0;
        assert!(diff >= 0, "second instant is later than self");
        Duration(diff as u32)
    }
}

#[cfg(feature = "timer-queue")]
impl ops::AddAssign<Duration> for Instant {
    fn add_assign(&mut self, dur: Duration) {
        debug_assert!(dur.0 < (1 << 31));
        self.0 = self.0.wrapping_add(dur.0 as i32);
    }
}

#[cfg(feature = "timer-queue")]
impl ops::Add<Duration> for Instant {
    type Output = Self;

    fn add(mut self, dur: Duration) -> Self {
        self += dur;
        self
    }
}

#[cfg(feature = "timer-queue")]
impl ops::SubAssign<Duration> for Instant {
    fn sub_assign(&mut self, dur: Duration) {
        // XXX should this be a non-debug assertion?
        debug_assert!(dur.0 < (1 << 31));
        self.0 = self.0.wrapping_sub(dur.0 as i32);
    }
}

#[cfg(feature = "timer-queue")]
impl ops::Sub<Duration> for Instant {
    type Output = Self;

    fn sub(mut self, dur: Duration) -> Self {
        self -= dur;
        self
    }
}

#[cfg(feature = "timer-queue")]
impl ops::Sub<Instant> for Instant {
    type Output = Duration;

    fn sub(self, other: Instant) -> Duration {
        self.duration_since(other)
    }
}

#[cfg(feature = "timer-queue")]
impl Ord for Instant {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.0.wrapping_sub(rhs.0).cmp(&0)
    }
}

#[cfg(feature = "timer-queue")]
impl PartialOrd for Instant {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}

/// A `Duration` type to represent a span of time.
///
/// This data type is only available when the `timer-queue` feature is enabled
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
#[cfg(feature = "timer-queue")]
pub struct Duration(u32);

#[cfg(feature = "timer-queue")]
impl ops::AddAssign for Duration {
    fn add_assign(&mut self, dur: Duration) {
        self.0 += dur.0;
    }
}

#[cfg(feature = "timer-queue")]
impl ops::Add<Duration> for Duration {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Duration(self.0 + other.0)
    }
}

#[cfg(feature = "timer-queue")]
impl ops::SubAssign for Duration {
    fn sub_assign(&mut self, rhs: Duration) {
        self.0 -= rhs.0;
    }
}

#[cfg(feature = "timer-queue")]
impl ops::Sub<Duration> for Duration {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Duration(self.0 - rhs.0)
    }
}

/// Adds the `cycles` method to the `u32` type
///
/// This trait is only available when the `timer-queue` feature is enabled
#[cfg(feature = "timer-queue")]
pub trait U32Ext {
    /// Converts the `u32` value into clock cycles
    fn cycles(self) -> Duration;
}

#[cfg(feature = "timer-queue")]
impl U32Ext for u32 {
    fn cycles(self) -> Duration {
        Duration(self)
    }
}

/// Memory safe access to shared resources
///
/// In RTFM, locks are implemented as critical sections that prevent other tasks from *starting*.
/// These critical sections are implemented by temporarily increasing the dynamic priority (see
/// [BASEPRI]) of the current context. Entering and leaving these critical sections is always done
/// in constant time (a few instructions).
///
/// [BASEPRI]: https://developer.arm.com/products/architecture/cpu-architecture/m-profile/docs/100701/latest/special-purpose-mask-registers
pub trait Mutex {
    /// Data protected by the mutex
    type T;

    /// Creates a critical section and grants temporary access to the protected data
    fn lock<R, F>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self::T) -> R;
}

impl<'a, M> Mutex for &'a mut M
where
    M: Mutex,
{
    type T = M::T;

    fn lock<R, F>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self::T) -> R,
    {
        (**self).lock(f)
    }
}

/// Newtype over `&'a mut T` that implements the `Mutex` trait
///
/// The `Mutex` implementation for this type is a no-op, no critical section is created
pub struct Exclusive<'a, T>(pub &'a mut T);

impl<'a, T> Mutex for Exclusive<'a, T> {
    type T = T;

    fn lock<R, F>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self::T) -> R,
    {
        f(self.0)
    }
}

impl<'a, T> fmt::Debug for Exclusive<'a, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<'a, T> fmt::Display for Exclusive<'a, T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<'a, T> ops::Deref for Exclusive<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.0
    }
}

impl<'a, T> ops::DerefMut for Exclusive<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0
    }
}

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
