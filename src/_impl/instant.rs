use core::cmp::Ordering;
use core::{ops, ptr};

use cortex_m::peripheral::DWT;

#[derive(Clone, Copy, Debug)]
pub struct Instant(pub u32);

impl Into<u32> for Instant {
    fn into(self) -> u32 {
        self.0
    }
}

impl Instant {
    pub fn now() -> Self {
        const DWT_CYCCNT: *const u32 = 0xE000_1004 as *const u32;

        // NOTE(ptr::read) don't use a volatile load to let the compiler optimize this away
        Instant(unsafe { ptr::read(DWT_CYCCNT) })
    }
}

impl Eq for Instant {}

impl Ord for Instant {
    fn cmp(&self, rhs: &Self) -> Ordering {
        (self.0 as i32).wrapping_sub(rhs.0 as i32).cmp(&0)
    }
}

impl PartialEq for Instant {
    fn eq(&self, rhs: &Self) -> bool {
        self.0.eq(&rhs.0)
    }
}

impl PartialOrd for Instant {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}

impl ops::Add<u32> for Instant {
    type Output = Self;

    fn add(self, rhs: u32) -> Self {
        Instant(self.0.wrapping_add(rhs))
    }
}

impl ops::Sub for Instant {
    type Output = i32;

    fn sub(self, rhs: Self) -> i32 {
        (self.0 as i32).wrapping_sub(rhs.0 as i32)
    }
}
