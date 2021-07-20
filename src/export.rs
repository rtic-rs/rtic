use core::{
    cell::Cell,
    sync::atomic::{AtomicBool, Ordering},
};

pub use crate::tq::{NotReady, TimerQueue};
pub use bare_metal::CriticalSection;
#[cfg(armv7m)]
pub use cortex_m::register::basepri;
pub use cortex_m::{
    asm::wfi,
    interrupt,
    peripheral::{scb::SystemHandler, syst::SystClkSource, DWT, NVIC},
    Peripherals,
};
pub use heapless::spsc::Queue;
pub use heapless::BinaryHeap;
pub use rtic_monotonic as monotonic;

pub type SCFQ<const N: usize> = Queue<u8, N>;
pub type SCRQ<T, const N: usize> = Queue<(T, u8), N>;

#[cfg(armv7m)]
#[inline(always)]
pub fn run<F>(priority: u8, f: F)
where
    F: FnOnce(),
{
    if priority == 1 {
        // If the priority of this interrupt is `1` then BASEPRI can only be `0`
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
    /// Create a new Priority
    ///
    /// # Safety
    ///
    /// Will overwrite the current Priority
    #[inline(always)]
    pub unsafe fn new(value: u8) -> Self {
        Priority {
            inner: Cell::new(value),
        }
    }

    /// Change the current priority to `value`
    // These two methods are used by `lock` (see below) but can't be used from the RTIC application
    #[inline(always)]
    fn set(&self, value: u8) {
        self.inner.set(value)
    }

    /// Get the current priority
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
pub fn assert_monotonic<T>()
where
    T: monotonic::Monotonic,
{
}

/// Lock the resource proxy by setting the BASEPRI
/// and running the closure with interrupt::free
///
/// # Safety
///
/// Writing to the BASEPRI
/// Dereferencing a raw pointer
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

/// Lock the resource proxy by setting the PRIMASK
/// and running the closure with interrupt::free
///
/// # Safety
///
/// Writing to the PRIMASK
/// Dereferencing a raw pointer
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
