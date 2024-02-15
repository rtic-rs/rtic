//! A channel operating on bitflags.

use bitflags::{Bits, Flags};
use core::sync::atomic::{AtomicU16, AtomicU8, Ordering};
use portable_atomic::{AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicU32, AtomicU64};

/// A channel for setting and clearing `bitflags` concurrently.
pub struct BitChannel<T: Flags>
where
    T: Flags,
    T::Bits: AtomicType,
{
    atomic: <T::Bits as AtomicType>::Atomic,
}

impl<T> BitChannel<T>
where
    T: Flags,
    T::Bits: AtomicType,
{
    /// Create a new `bitflags` channel.
    pub const fn new() -> Self {
        BitChannel {
            atomic: T::Bits::ATOMIC_ZERO,
        }
    }

    /// Set `bitflag`s.
    #[inline]
    pub fn send(&self, flags: T) {
        T::Bits::fetch_or(&self.atomic, flags.bits(), Ordering::Relaxed);
    }

    /// Receive the current value of the `bitflags` and reset all flags.
    #[inline]
    pub fn recv(&self) -> T {
        <T as Flags>::from_bits_retain(T::Bits::fetch_and(
            &self.atomic,
            T::Bits::EMPTY,
            Ordering::Relaxed,
        ))
    }
}

/// Generic atomic trait, allows for taking any `bitflags::Bits` as an atomic.
pub trait AtomicType: Sized {
    /// The underlying atomic, e.g. `AtomicU8`, `AtomicU16`.
    type Atomic: From<Self>;

    /// The definition of that atomic with value 0.
    const ATOMIC_ZERO: Self::Atomic;

    /// The atomic's `fetch_and` implementation forwarded.
    fn fetch_and(a: &Self::Atomic, b: Self, order: Ordering) -> Self;

    /// The atomic's `fetch_or` implementation forwarded.
    fn fetch_or(a: &Self::Atomic, b: Self, order: Ordering) -> Self;
}

macro_rules! atomic_type_impl {
    ($atomic:ty, $integer:ty) => {
        impl AtomicType for $integer {
            type Atomic = $atomic;

            const ATOMIC_ZERO: Self::Atomic = <$atomic>::new(0);

            #[inline(always)]
            fn fetch_and(a: &Self::Atomic, b: Self, order: Ordering) -> Self {
                a.fetch_and(b, order)
            }

            #[inline(always)]
            fn fetch_or(a: &Self::Atomic, b: Self, order: Ordering) -> Self {
                a.fetch_or(b, order)
            }
        }
    };
}

atomic_type_impl!(AtomicU8, u8);
atomic_type_impl!(AtomicU16, u16);
atomic_type_impl!(AtomicU32, u32);
atomic_type_impl!(AtomicU64, u64);
atomic_type_impl!(AtomicI8, i8);
atomic_type_impl!(AtomicI16, i16);
atomic_type_impl!(AtomicI32, i32);
atomic_type_impl!(AtomicI64, i64);

// etc...

#[cfg(test)]
mod tests {
    use super::*;
    use bitflags::bitflags;

    bitflags! {
        struct FlagsU8: u8 {
            const A = 1;
            const B = 2;
        }
    }

    bitflags! {
        struct FlagsU16: u16 {
            const A = 1;
            const B = 2;
        }
    }

    #[test]
    fn test() {
        let a = BitChannel::<FlagsU8>::new();
        let b = BitChannel::<FlagsU16>::new();
    }
}
