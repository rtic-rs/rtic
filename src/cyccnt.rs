//! Data Watchpoint Trace (DWT) unit's CYCle CouNTer (CYCCNT)

use cortex_m::peripheral::DWT;

use crate::time::{self, instant::Instant, Ratio};

/// Implementation of the `Monotonic` trait based on CYCle CouNTer
pub struct CYCCNT;

impl crate::Monotonic for CYCCNT {
    unsafe fn reset() {
        (0xE0001004 as *mut u32).write_volatile(0)
    }
}

impl time::Clock for CYCCNT {
    type Rep = i32;

    fn now() -> Instant<Self>
    {
        let ticks = DWT::get_cycle_count();

        Instant::new(ticks as Self::Rep)
    }
}

impl time::Period for CYCCNT {
    const PERIOD: Ratio<i32> = Ratio::new_raw(1, 64_000_000);
}
