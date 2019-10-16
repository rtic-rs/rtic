//! Data Watchpoint Trace (DWT) unit's CYCle CouNTer (CYCCNT)

use core::{
    cmp::Ordering,
    convert::{Infallible, TryInto},
    fmt, ops,
};

use cortex_m::peripheral::DWT;

use crate::Fraction;

/// A measurement of the CYCCNT. Opaque and useful only with `Duration`
///
/// This data type is only available on ARMv7-M
///
/// # Correctness
///
/// Adding or subtracting a `Duration` of more than `(1 << 31)` cycles to an `Instant` effectively
/// makes it "wrap around" and creates an incorrect value. This is also true if the operation is
/// done in steps, e.g. `(instant + dur) + dur` where `dur` is `(1 << 30)` ticks.
///
/// In multi-core contexts: this value is tied to the CYCCNT of *one* core so sending it a different
/// core makes it lose its meaning -- each Cortex-M core has its own CYCCNT counter and these are
/// usually unsynchronized and may even be running at different frequencies.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Instant {
    inner: i32,
}

impl Instant {
    /// Returns an instant corresponding to "now"
    pub fn now() -> Self {
        Instant {
            inner: DWT::get_cycle_count() as i32,
        }
    }

    /// Returns the amount of time elapsed since this instant was created.
    pub fn elapsed(&self) -> Duration {
        Instant::now() - *self
    }

    /// Returns the amount of time elapsed from another instant to this one.
    pub fn duration_since(&self, earlier: Instant) -> Duration {
        let diff = self.inner - earlier.inner;
        assert!(diff >= 0, "second instant is later than self");
        Duration { inner: diff as u32 }
    }
}

impl fmt::Debug for Instant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Instant")
            .field(&(self.inner as u32))
            .finish()
    }
}

impl ops::AddAssign<Duration> for Instant {
    fn add_assign(&mut self, dur: Duration) {
        // NOTE this is a debug assertion because there's no foolproof way to detect a wrap around;
        // the user may write `(instant + dur) + dur` where `dur` is `(1<<31)-1` ticks.
        debug_assert!(dur.inner < (1 << 31));
        self.inner = self.inner.wrapping_add(dur.inner as i32);
    }
}

impl ops::Add<Duration> for Instant {
    type Output = Self;

    fn add(mut self, dur: Duration) -> Self {
        self += dur;
        self
    }
}

impl ops::SubAssign<Duration> for Instant {
    fn sub_assign(&mut self, dur: Duration) {
        // NOTE see the NOTE in `<Instant as AddAssign<Duration>>::add_assign`
        debug_assert!(dur.inner < (1 << 31));
        self.inner = self.inner.wrapping_sub(dur.inner as i32);
    }
}

impl ops::Sub<Duration> for Instant {
    type Output = Self;

    fn sub(mut self, dur: Duration) -> Self {
        self -= dur;
        self
    }
}

impl ops::Sub<Instant> for Instant {
    type Output = Duration;

    fn sub(self, other: Instant) -> Duration {
        self.duration_since(other)
    }
}

impl Ord for Instant {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.inner.wrapping_sub(rhs.inner).cmp(&0)
    }
}

impl PartialOrd for Instant {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}

/// A `Duration` type to represent a span of time.
///
/// This data type is only available on ARMv7-M
///
/// # Correctness
///
/// This type is *not* appropriate for representing time spans in the order of, or larger than,
/// seconds because it can hold a maximum of `(1 << 31)` "ticks" where each tick is the inverse of
/// the CPU frequency, which usually is dozens of MHz.
#[derive(Clone, Copy, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Duration {
    inner: u32,
}

impl Duration {
    /// Creates a new `Duration` from the specified number of clock cycles
    pub fn from_cycles(cycles: u32) -> Self {
        Duration { inner: cycles }
    }

    /// Returns the total number of clock cycles contained by this `Duration`
    pub fn as_cycles(&self) -> u32 {
        self.inner
    }
}

impl TryInto<u32> for Duration {
    type Error = Infallible;

    fn try_into(self) -> Result<u32, Infallible> {
        Ok(self.as_cycles())
    }
}

impl ops::AddAssign for Duration {
    fn add_assign(&mut self, dur: Duration) {
        self.inner += dur.inner;
    }
}

impl ops::Add<Duration> for Duration {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Duration {
            inner: self.inner + other.inner,
        }
    }
}

impl ops::SubAssign for Duration {
    fn sub_assign(&mut self, rhs: Duration) {
        self.inner -= rhs.inner;
    }
}

impl ops::Sub<Duration> for Duration {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Duration {
            inner: self.inner - rhs.inner,
        }
    }
}

/// Adds the `cycles` method to the `u32` type
///
/// This trait is only available on ARMv7-M
pub trait U32Ext {
    /// Converts the `u32` value into clock cycles
    fn cycles(self) -> Duration;
}

impl U32Ext for u32 {
    fn cycles(self) -> Duration {
        Duration { inner: self }
    }
}

/// Implementation of the `Monotonic` trait based on CYCle CouNTer
pub struct CYCCNT;

impl crate::Monotonic for CYCCNT {
    type Instant = Instant;

    fn ratio() -> Fraction {
        Fraction {
            numerator: 1,
            denominator: 1,
        }
    }

    unsafe fn reset() {
        (0xE0001004 as *mut u32).write_volatile(0)
    }

    fn now() -> Instant {
        Instant::now()
    }

    fn zero() -> Instant {
        Instant { inner: 0 }
    }
}
