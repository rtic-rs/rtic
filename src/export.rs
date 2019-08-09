//! IMPLEMENTATION DETAILS. DO NOT USE ANYTHING IN THIS MODULE

use core::{cell::Cell, u8};

#[cfg(armv7m)]
use cortex_m::register::basepri;
pub use cortex_m::{
    asm::wfi, interrupt, peripheral::scb::SystemHandler, peripheral::syst::SystClkSource,
    peripheral::Peripherals,
};
pub use heapless::consts;
use heapless::spsc::{Queue, SingleCore};

#[cfg(feature = "timer-queue")]
pub use crate::tq::{isr as sys_tick, NotReady, TimerQueue};

pub type FreeQueue<N> = Queue<u8, N, usize, SingleCore>;
pub type ReadyQueue<T, N> = Queue<(T, u8), N, usize, SingleCore>;

#[cfg(armv7m)]
#[inline(always)]
pub fn run<F>(f: F)
where
    F: FnOnce(),
{
    let initial = basepri::read();
    f();
    unsafe { basepri::write(initial) }
}

#[cfg(not(armv7m))]
#[inline(always)]
pub fn run<F>(f: F)
where
    F: FnOnce(),
{
    f();
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

    // these two methods are used by claim (see below) but can't be used from the RTFM application
    #[inline(always)]
    fn set(&self, value: u8) {
        self.inner.set(value)
    }

    #[inline(always)]
    fn get(&self) -> u8 {
        self.inner.get()
    }
}

pub struct MaybeUninit<T> {
    // we newtype so the end-user doesn't need `#![feature(maybe_uninit)]` in their code
    inner: core::mem::MaybeUninit<T>,
}

impl<T> MaybeUninit<T> {
    pub const fn uninit() -> Self {
        MaybeUninit {
            inner: core::mem::MaybeUninit::uninit(),
        }
    }

    pub fn as_ptr(&self) -> *const T {
        self.inner.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.inner.as_mut_ptr()
    }

    #[cfg(feature = "nightly")]
    pub fn write(&mut self, value: T) -> &mut T {
        self.inner.write(value)
    }

    #[cfg(not(feature = "nightly"))]
    pub unsafe fn get_ref(&self) -> &T {
        &*self.inner.as_ptr()
    }

    #[cfg(not(feature = "nightly"))]
    pub unsafe fn get_mut(&mut self) -> &mut T {
        &mut *self.inner.as_mut_ptr()
    }

    #[cfg(not(feature = "nightly"))]
    pub fn write(&mut self, value: T) -> &mut T {
        self.inner = core::mem::MaybeUninit::new(value);
        unsafe { self.get_mut() }
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

#[cfg(armv7m)]
#[inline(always)]
pub unsafe fn claim<T, R, F>(
    ptr: *mut T,
    priority: &Priority,
    ceiling: u8,
    nvic_prio_bits: u8,
    f: F,
) -> R
where
    F: FnOnce(&mut T) -> R,
{
    let current = priority.get();

    if priority.get() < ceiling {
        if ceiling == (1 << nvic_prio_bits) {
            priority.set(u8::MAX);
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
pub unsafe fn claim<T, R, F>(
    ptr: *mut T,
    priority: &Priority,
    ceiling: u8,
    _nvic_prio_bits: u8,
    f: F,
) -> R
where
    F: FnOnce(&mut T) -> R,
{
    let current = priority.get();

    if priority.get() < ceiling {
        priority.set(u8::MAX);
        let r = interrupt::free(|_| f(&mut *ptr));
        priority.set(current);
        r
    } else {
        f(&mut *ptr)
    }
}

#[cfg(armv7m)]
#[inline]
fn logical2hw(logical: u8, nvic_prio_bits: u8) -> u8 {
    ((1 << nvic_prio_bits) - logical) << (8 - nvic_prio_bits)
}
