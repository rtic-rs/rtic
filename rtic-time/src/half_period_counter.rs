//! Utility to implement a race condition free half-period based monotonic.
//!
//! # Background
//!
//! Monotonics are continuous and never wrap (in a reasonable amount of time), while
//! the underlying hardware usually wraps frequently and has interrupts to indicate that
//! a wrap happened.
//!
//! The biggest problem when implementing a monotonic from such hardware is that there exists
//! a non-trivial race condition while reading data from the timer. Let's assume we increment
//! a period counter every time an overflow interrupt happens.
//! Which should we then read first when computing the current time? The period counter or
//! the timer value?
//! - When reading the timer value first, an overflow interrupt could happen before we read
//!   the period counter, causing the calculated time to be much too high
//! - When reading the period counter first, an the timer value could overflow before we
//!   read it, casing the caluclated time to be much too low
//!
//! The reason this is non-trivil to solve is because even critical sections do not help
//! much - the inherent problem here is that the timer value continues to change, and there
//! is no way to read it together with the period counter in an atomic way.
//!
//! # Solution
//!
//! This module provides utilities to solve this problem in a reliable, race-condition free way.
//! A second interrupt must be added at the half-period mark, which effectively converts the period counter
//! to a half-period counter. This creates one bit of overlap between the
//! timer value and the period counter, which makes it mathematically possible to solve the
//! race condition.
//!
//! The following steps have to be fulfilled to make this reliable:
//! - The period counter gets incremented twice per period; once when the timer overflow happens and once
//!   at the half-period mark. For example, a 16-bit timer would require the period counter to be
//!   incremented at the values `0x0000` and `0x8000`.
//! - The timer value and the period counter must be in sync. After the overflow interrupt
//!   was processed, the period counter must be even, and after the half-way interrupt was
//!   processed, the period counter must be odd.
//! - Both the overflow interrupt and the half-way interrupt must be processed within half a
//!   timer period. This means those interrupts should be the highest priority in the
//!   system - disabling them for more than half a period will cause the monotonic to misbehave.
//!
//! If those conditions are fulfilled, the [`calculate_now`] function will reliably always
//! return the current time.
//!
//! # Why does this work?
//!
//! It's complicated. In essence, this one bit of overlap can be used mathematically to make
//! it irrelevant whether the period counter was already incremented or not.
//! For example, during the second part of the timer period, it is irrelevant if the
//! period counter is `2` (before the interrupt) or `3` (after the interrupt) - [`calculate_now`]
//! will yield the same result. Then half a period later, in the first part of the next timer period,
//! it is irrelevant if the period counter is `3` or `4` - they again will yield the same result.
//!
//! This means that as long as we read the period counter **before** the timer value, we will
//! always get the correct result, given that the interrupts are not delayed by more than half a period.
//!
//! # Example
//!
//! This example assumes that the underlying timer is 16-bit.
//!
//! ```rust
//! static HALF_PERIOD_COUNTER: AtomicU32 = AtomicU32::new(0);
//!
//! struct MyMonotonic;
//!
//! impl MyMonotonic {
//!     fn init() {
//!         timer_stop();
//!         timer_reset();
//!         HALF_PERIOD_COUNTER.store(0, Relaxed);
//!         timer_enable_overflow_interrupt();
//!         timer_enable_compare_interrupt(0x8000);
//!         // The period counter is reset to zero, the timer is reset
//!         // and the overflow is enabled. This means the period counter
//!         // and the timer value are in sync, so we can now enable the
//!         // timer.
//!         timer_start();
//!     }
//!
//!     fn on_interrupt() {
//!         if overflow_interrupt_happened() {
//!             clear_overflow_interrupt();
//!             HALF_PERIOD_COUNTER.fetch_add(1, Relaxed);
//!         }
//!         if compare_interrupt_happened() {
//!             clear_compare_interrupt();
//!             HALF_PERIOD_COUNTER.fetch_add(1, Relaxed);
//!         }
//!     }
//!
//!     fn now() -> u64 {
//!         rtic_time::half_period_counter::calculate_now(
//!             &HALF_PERIOD_COUNTER,
//!             || timer_get_value(),
//!         )
//!     }
//! }
//! ```
//!

use atomic_polyfill::{compiler_fence, AtomicU16, AtomicU32, AtomicU64, AtomicU8, Ordering};

/// A half period overflow counter.
pub trait HalfPeriods {
    /// The type of the stored value.
    type Inner: Copy;
    /// Retrieves the stored value. Can use `Ordering::Relaxed`.
    fn load_relaxed(&self) -> Self::Inner;
}
macro_rules! impl_half_periods {
    ($ta:ty, $t:ty) => {
        impl HalfPeriods for $ta {
            type Inner = $t;
            #[inline]
            fn load_relaxed(&self) -> Self::Inner {
                self.load(Ordering::Relaxed)
            }
        }
    };
}
impl_half_periods!(AtomicU8, u8);
impl_half_periods!(AtomicU16, u16);
impl_half_periods!(AtomicU32, u32);
impl_half_periods!(AtomicU64, u64);

/// The value of the timer's count register.
pub trait TimerValue {
    /// Bit size of the register.
    const BITS: u32;
}
macro_rules! impl_timer_value {
    ($t:ty) => {
        impl TimerValue for $t {
            const BITS: u32 = Self::BITS;
        }
    };
}
impl_timer_value!(u8);
impl_timer_value!(u16);
impl_timer_value!(u32);
impl_timer_value!(u64);

/// Operations a type has to support
/// in order to be used as the return value
/// of [`calculate_now`].
pub trait TimerOps: Copy {
    /// All bits set to `1`.
    const MAX: Self;
    /// The lowest bit set to `1`.
    const ONE: Self;
    /// The `^` operation.
    fn xor(self, other: Self) -> Self;
    /// The `&` operation.
    fn and(self, other: Self) -> Self;
    /// The `+` operation.
    fn add(self, other: Self) -> Self;
    /// The `<<` operation.
    fn left_shift(self, amount: u32) -> Self;
}

macro_rules! impl_timer_ops {
    ($t:ty) => {
        impl TimerOps for $t {
            const MAX: Self = Self::MAX;
            const ONE: Self = 1;

            #[inline]
            fn xor(self, other: Self) -> Self {
                self ^ other
            }

            #[inline]
            fn and(self, other: Self) -> Self {
                self & other
            }

            #[inline]
            fn add(self, other: Self) -> Self {
                self + other
            }

            #[inline]
            fn left_shift(self, amount: u32) -> Self {
                self << amount
            }
        }
    };
}

impl_timer_ops!(u16);
impl_timer_ops!(u32);
impl_timer_ops!(u64);
impl_timer_ops!(u128);

/// Calculates the current time from the half period counter and the current timer value.
///
/// # Arguments
///
/// * `half_periods` - A reference to the atomic counter that stores the half period count.
/// * `timer_value` - A closure/function that when called produces the current timer value.
pub fn calculate_now<P, T, F, O>(half_periods: &P, timer_value: F) -> O
where
    P: HalfPeriods,
    T: TimerValue,
    O: From<P::Inner> + From<T> + TimerOps,
    F: FnOnce() -> T,
{
    // Important: half_period **must** be read first.
    // Otherwise we have another mathematical race condition.
    let half_periods = O::from(half_periods.load_relaxed());
    compiler_fence(Ordering::Acquire);
    let timer_value = O::from(timer_value());

    // Credits to the `time-driver` of `embassy-stm32`.
    //
    // Given that our clock counter value is 32 bits.
    //
    // Clock timekeeping works with something we call "periods", which are time intervals
    // of 2^31 ticks. The Clock counter value is 32 bits, so one "overflow cycle" is 2 periods.
    //
    // A `period` count is maintained in parallel to the Timer hardware `counter`, like this:
    // - `period` and `counter` start at 0
    // - `period` is incremented on overflow (at counter value 0)
    // - `period` is incremented "midway" between overflows (at counter value 0x8000_0000)
    //
    // Therefore, when `period` is even, counter is in 0..0x7FFF_FFFF. When odd, counter is in 0x8000_0000..0xFFFF_FFFF
    // This allows for now() to return the correct value even if it races an overflow.
    //
    // To get `now()`, `period` is read first, then `counter` is read. If the counter value matches
    // the expected range for the `period` parity, we're done. If it doesn't, this means that
    // a new period start has raced us between reading `period` and `counter`, so we assume the `counter` value
    // corresponds to the next period.
    //
    // `period` is a 32bit integer, so it overflows on 2^32 * 2^31 / 1_000_000 seconds of uptime, which is 292471 years.

    // Formula:
    //   (half_periods << (timer_value::BITS - 1))
    //   + u64::from(
    //        timer_value ^ (
    //            ((half_periods & 1) as timer_value::TYPE) << (timer_value::BITS - 1)
    //        )
    //     )

    let upper_half = half_periods.left_shift(T::BITS - 1);
    let lower_half = O::ONE.left_shift(T::BITS - 1).and(upper_half);
    upper_half.add(lower_half.xor(timer_value))
}
