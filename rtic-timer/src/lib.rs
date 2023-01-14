#![no_std]

use core::sync::atomic::{AtomicU32, Ordering};
use core::{cmp::Ordering, task::Waker};
use cortex_m::peripheral::{syst::SystClkSource, SYST};
pub use fugit::{self, ExtU64};
pub use rtic_monotonic::Monotonic;

mod sll;
use sll::{IntrusiveSortedLinkedList, Min as IsslMin, Node as IntrusiveNode};

pub struct Timer {
    cnt: AtomicU32,
    // queue: IntrusiveSortedLinkedList<'static, WakerNotReady<Mono>, IsslMin>,
}

#[allow(non_snake_case)]
#[no_mangle]
fn SysTick() {
    // ..
    let cnt = unsafe {
        static mut CNT: u32 = 0;
        &mut CNT
    };

    *cnt = cnt.wrapping_add(1);
}

/// Systick implementing `rtic_monotonic::Monotonic` which runs at a
/// settable rate using the `TIMER_HZ` parameter.
pub struct Systick<const TIMER_HZ: u32> {
    systick: SYST,
    cnt: u64,
}

impl<const TIMER_HZ: u32> Systick<TIMER_HZ> {
    /// Provide a new `Monotonic` based on SysTick.
    ///
    /// The `sysclk` parameter is the speed at which SysTick runs at. This value should come from
    /// the clock generation function of the used HAL.
    ///
    /// Notice that the actual rate of the timer is a best approximation based on the given
    /// `sysclk` and `TIMER_HZ`.
    pub fn new(mut systick: SYST, sysclk: u32) -> Self {
        // + TIMER_HZ / 2 provides round to nearest instead of round to 0.
        // - 1 as the counter range is inclusive [0, reload]
        let reload = (sysclk + TIMER_HZ / 2) / TIMER_HZ - 1;

        assert!(reload <= 0x00ff_ffff);
        assert!(reload > 0);

        systick.disable_counter();
        systick.set_clock_source(SystClkSource::Core);
        systick.set_reload(reload);

        Systick { systick, cnt: 0 }
    }
}

impl<const TIMER_HZ: u32> Monotonic for Systick<TIMER_HZ> {
    const DISABLE_INTERRUPT_ON_EMPTY_QUEUE: bool = false;

    type Instant = fugit::TimerInstantU64<TIMER_HZ>;
    type Duration = fugit::TimerDurationU64<TIMER_HZ>;

    fn now(&mut self) -> Self::Instant {
        if self.systick.has_wrapped() {
            self.cnt = self.cnt.wrapping_add(1);
        }

        Self::Instant::from_ticks(self.cnt)
    }

    unsafe fn reset(&mut self) {
        self.systick.clear_current();
        self.systick.enable_counter();
    }

    #[inline(always)]
    fn set_compare(&mut self, _val: Self::Instant) {
        // No need to do something here, we get interrupts anyway.
    }

    #[inline(always)]
    fn clear_compare_flag(&mut self) {
        // NOOP with SysTick interrupt
    }

    #[inline(always)]
    fn zero() -> Self::Instant {
        Self::Instant::from_ticks(0)
    }

    #[inline(always)]
    fn on_interrupt(&mut self) {
        if self.systick.has_wrapped() {
            self.cnt = self.cnt.wrapping_add(1);
        }
    }
}

struct WakerNotReady<Mono>
where
    Mono: Monotonic,
{
    pub waker: Waker,
    pub instant: Mono::Instant,
    pub marker: u32,
}

impl<Mono> Eq for WakerNotReady<Mono> where Mono: Monotonic {}

impl<Mono> Ord for WakerNotReady<Mono>
where
    Mono: Monotonic,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.instant.cmp(&other.instant)
    }
}

impl<Mono> PartialEq for WakerNotReady<Mono>
where
    Mono: Monotonic,
{
    fn eq(&self, other: &Self) -> bool {
        self.instant == other.instant
    }
}

impl<Mono> PartialOrd for WakerNotReady<Mono>
where
    Mono: Monotonic,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
