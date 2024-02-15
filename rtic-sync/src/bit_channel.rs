//! A channel operating on bitflags.

use bitflags::{Bits, Flags};
use core::sync::atomic::{AtomicU16, AtomicU8, Ordering};

//
// Bit channel
//

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
    pub const fn new() -> Self {
        BitChannel {
            atomic: T::Bits::ATOMIC_ZERO,
        }
    }

    pub fn test(&self) {
        T::Bits::fetch_and(&self.atomic, T::Bits::EMPTY, Ordering::Relaxed);
    }
}

pub trait AtomicType: Sized {
    type Atomic: From<Self>;
    
    const ATOMIC_ZERO: Self::Atomic;

    fn fetch_and(a: &Self::Atomic, b: Self, order: Ordering) -> Self;
}

impl AtomicType for u16 {
    type Atomic = AtomicU16;
    
    const ATOMIC_ZERO: Self::Atomic = AtomicU16::new(0);

    fn fetch_and(a: &Self::Atomic, b: Self, order: Ordering) -> Self {
        a.fetch_and(b, order)
    }
}

impl AtomicType for u8 {
    type Atomic = AtomicU8;

    const ATOMIC_ZERO: Self::Atomic = AtomicU8::new(0);

    fn fetch_and(a: &Self::Atomic, b: Self, order: Ordering) -> Self {
        a.fetch_and(b, order)
    }
}

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

