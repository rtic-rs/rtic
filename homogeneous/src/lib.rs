//! Fake multi-core PAC

#![no_std]

use bare_metal::Nr;
use rtic::{time, Monotonic, MultiCore};

// both cores have the exact same interrupts
pub use Interrupt_0 as Interrupt_1;

// Fake priority bits
pub const NVIC_PRIO_BITS: u8 = 3;

pub fn xpend(_core: u8, _interrupt: impl Nr) {}

/// Fake monotonic timer
pub struct MT;

impl time::Clock for MT {
    type Rep = i32;
    const PERIOD: time::Period<i32> = time::Period::new(1, 1);

    fn now() -> time::Instant<Self> {
        unsafe { time::Instant::new((0xE0001004 as *const u32).read_volatile() as i32) }
    }
}

impl Monotonic for MT {
    unsafe fn reset() {
        (0xE0001004 as *mut u32).write_volatile(0)
    }
}

impl MultiCore for MT {}

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
