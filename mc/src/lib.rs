//! Fake multi-core PAC

#![no_std]

use core::{
    cmp::Ordering,
    ops::{Add, Sub},
};

use cortex_m::interrupt::Nr;
use rtfm::Monotonic;

// Fake priority bits
pub const NVIC_PRIO_BITS: u8 = 3;

pub struct CrossPend;

pub fn xpend(_core: u8, _interrupt: impl Nr) {}

/// Fake monotonic timer
pub struct MT;

unsafe impl Monotonic for MT {
    type Instant = Instant;

    fn ratio() -> u32 {
        1
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
pub enum Interrupt {
    I0,
    I1,
    I2,
    I3,
    I4,
    I5,
    I6,
    I7,
}

unsafe impl Nr for Interrupt {
    fn nr(&self) -> u8 {
        match self {
            Interrupt::I0 => 0,
            Interrupt::I1 => 1,
            Interrupt::I2 => 2,
            Interrupt::I3 => 3,
            Interrupt::I4 => 4,
            Interrupt::I5 => 5,
            Interrupt::I6 => 6,
            Interrupt::I7 => 7,
        }
    }
}
