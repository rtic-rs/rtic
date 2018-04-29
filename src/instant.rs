use core::cmp::Ordering;
use core::ops;

use cortex_m::peripheral::DWT;

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct Instant(u32);

impl Into<u32> for Instant {
    fn into(self) -> u32 {
        self.0
    }
}

impl Instant {
    pub unsafe fn new(timestamp: u32) -> Self {
        Instant(timestamp)
    }

    pub fn now() -> Self {
        Instant(DWT::get_cycle_count())
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
