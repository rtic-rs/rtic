//! Fake multi-core PAC

#![no_std]

use core::{
    cmp::Ordering,
    ops::{Add, Sub},
};

use bare_metal::Nr;
use rtfm::{Fraction, Monotonic, MultiCore};

// both cores have the exact same interrupts
pub use Interrupt_0 as Interrupt_1;

// Fake priority bits
pub const NVIC_PRIO_BITS: u8 = 3;

pub fn xpend(_core: u8, _interrupt: impl Nr) {}

/// Fake monotonic timer
pub struct MT;

impl Monotonic for MT {
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
        unsafe { Instant((0xE0001004 as *const u32).read_volatile() as i32) }
    }

    fn zero() -> Instant {
        Instant(0)
    }
}

impl MultiCore for MT {}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Instant(i32);

impl Add<u32> for Instant {
    type Output = Instant;

    fn add(self, rhs: u32) -> Self {
        Instant(self.0.wrapping_add(rhs as i32))
    }
}

impl Sub for Instant {
    type Output = u32;

    fn sub(self, rhs: Self) -> u32 {
        self.0.checked_sub(rhs.0).unwrap() as u32
    }
}

impl Ord for Instant {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.0.wrapping_sub(rhs.0).cmp(&0)
    }
}

impl PartialOrd for Instant {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}

// Fake interrupts
#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Interrupt_0 {
    I0 = 0,
    I1 = 1,
    I2 = 2,
    I3 = 3,
    I4 = 4,
    I5 = 5,
    I6 = 6,
    I7 = 7,
}

unsafe impl Nr for Interrupt_0 {
    fn nr(&self) -> u8 {
        *self as u8
    }
}
