//! Utilities to implement a race condition free half-period based monotonic.
//!
//! TODO: more detailed usage guide here

use atomic_polyfill::{compiler_fence, AtomicU16, AtomicU32, AtomicU64, AtomicU8, Ordering};

/// A half period overflow counter.
pub trait HalfPeriods {
    /// The type of the stored value.
    type Inner: Copy;
    /// Retreives the stored value.
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
    let now = upper_half.add(lower_half.xor(timer_value));

    now
}
