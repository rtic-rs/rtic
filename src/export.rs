use core::{
    cell::Cell,
    sync::atomic::{AtomicBool, Ordering},
};

pub use crate::tq::{NotReady, TimerQueue};
#[cfg(armv7m)]
pub use cortex_m::register::basepri;
pub use cortex_m::{
    asm::wfi,
    interrupt,
    peripheral::{scb::SystemHandler, syst::SystClkSource, DWT, NVIC},
    Peripherals,
};
use heapless::spsc::{MultiCore, SingleCore};
pub use heapless::{consts, i::Queue as iQueue, spsc::Queue};
pub use heapless::{i::BinaryHeap as iBinaryHeap, BinaryHeap};
#[cfg(feature = "heterogeneous")]
pub use microamp::shared;

pub type MCFQ<N> = Queue<u8, N, u8, MultiCore>;
pub type MCRQ<T, N> = Queue<(T, u8), N, u8, MultiCore>;
pub type SCFQ<N> = Queue<u8, N, u8, SingleCore>;
pub type SCRQ<T, N> = Queue<(T, u8), N, u8, SingleCore>;

#[cfg(armv7m)]
#[inline(always)]
pub fn run<F>(priority: u8, f: F)
where
    F: FnOnce(),
{
    if priority == 1 {
        // if the priority of this interrupt is `1` then BASEPRI can only be `0`
        f();
        unsafe { basepri::write(0) }
    } else {
        let initial = basepri::read();
        f();
        unsafe { basepri::write(initial) }
    }
}

#[cfg(not(armv7m))]
#[inline(always)]
pub fn run<F>(_priority: u8, f: F)
where
    F: FnOnce(),
{
    f();
}

pub struct Barrier {
    inner: AtomicBool,
}

impl Barrier {
    pub const fn new() -> Self {
        Barrier {
            inner: AtomicBool::new(false),
        }
    }

    pub fn release(&self) {
        self.inner.store(true, Ordering::Release)
    }

    pub fn wait(&self) {
        while !self.inner.load(Ordering::Acquire) {}
    }
}

// Newtype over `Cell` that forbids mutation through a shared reference
pub struct Priority {
    inner: Cell<u8>,
}

impl Priority {
    #[inline(always)]
    pub unsafe fn new(value: u8) -> Self {
        Priority {
            inner: Cell::new(value),
        }
    }

    // these two methods are used by `lock` (see below) but can't be used from the RTFM application
    #[inline(always)]
    fn set(&self, value: u8) {
        self.inner.set(value)
    }

    #[inline(always)]
    fn get(&self) -> u8 {
        self.inner.get()
    }
}

#[inline(always)]
pub fn assert_send<T>()
where
    T: Send,
{
}

#[inline(always)]
pub fn assert_sync<T>()
where
    T: Sync,
{
}

#[inline(always)]
pub fn assert_multicore<T>()
where
    T: super::MultiCore,
{
}

#[cfg(armv7m)]
#[inline(always)]
pub unsafe fn lock<T, R>(
    ptr: *mut T,
    priority: &Priority,
    ceiling: u8,
    nvic_prio_bits: u8,
    f: impl FnOnce(&mut T) -> R,
) -> R {
    let current = priority.get();

    if current < ceiling {
        if ceiling == (1 << nvic_prio_bits) {
            priority.set(u8::max_value());
            let r = interrupt::free(|_| f(&mut *ptr));
            priority.set(current);
            r
        } else {
            priority.set(ceiling);
            basepri::write(logical2hw(ceiling, nvic_prio_bits));
            let r = f(&mut *ptr);
            basepri::write(logical2hw(current, nvic_prio_bits));
            priority.set(current);
            r
        }
    } else {
        f(&mut *ptr)
    }
}

#[cfg(not(armv7m))]
#[inline(always)]
pub unsafe fn lock<T, R>(
    ptr: *mut T,
    priority: &Priority,
    ceiling: u8,
    _nvic_prio_bits: u8,
    f: impl FnOnce(&mut T) -> R,
) -> R {
    let current = priority.get();

    if current < ceiling {
        priority.set(u8::max_value());
        let r = interrupt::free(|_| f(&mut *ptr));
        priority.set(current);
        r
    } else {
        f(&mut *ptr)
    }
}

#[inline]
pub fn logical2hw(logical: u8, nvic_prio_bits: u8) -> u8 {
    ((1 << nvic_prio_bits) - logical) << (8 - nvic_prio_bits)
}
