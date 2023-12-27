use core::cmp;

/// The ticks of a timer.
pub trait TimerQueueTicks: Copy + PartialEq + Eq {
    /// Represents a single tick.
    const ONE_TICK: Self;

    /// Compares to another tick count.
    ///
    /// Takes into account timer wrapping; if the difference is more than
    /// half the value range, the result will be flipped.
    fn compare(self, other: Self) -> cmp::Ordering;

    /// True if `self` is at the same time as `other` or later.
    ///
    /// Takes into account timer wrapping; if the difference is more than
    /// half the value range, the result will be negated.
    fn is_at_least(self, other: Self) -> bool {
        match self.compare(other) {
            cmp::Ordering::Less => false,
            cmp::Ordering::Equal => true,
            cmp::Ordering::Greater => true,
        }
    }

    /// Wrapping addition.
    fn wrapping_add(self, other: Self) -> Self;
}

impl TimerQueueTicks for u32 {
    const ONE_TICK: Self = 1;

    fn compare(self, other: Self) -> cmp::Ordering {
        (self.wrapping_sub(other) as i32).cmp(&0)
    }
    fn wrapping_add(self, other: Self) -> Self {
        u32::wrapping_add(self, other)
    }
}
impl TimerQueueTicks for u64 {
    const ONE_TICK: Self = 1;

    fn compare(self, other: Self) -> cmp::Ordering {
        (self.wrapping_sub(other) as i64).cmp(&0)
    }
    fn wrapping_add(self, other: Self) -> Self {
        u64::wrapping_add(self, other)
    }
}
