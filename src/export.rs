//! IMPLEMENTATION DETAILS. DO NOT USE ANYTHING IN THIS MODULE

#[cfg(not(debug_assertions))]
use core::hint;
use core::{cell::Cell, ptr, u8};

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

// TODO(MaybeUninit) Until core::mem::MaybeUninit is stabilized we use our own (inefficient)
// implementation
pub struct MaybeUninit<T> {
    value: Option<T>,
}

impl<T> MaybeUninit<T> {
    pub const fn uninitialized() -> Self {
        MaybeUninit { value: None }
    }

    pub unsafe fn get_ref(&self) -> &T {
        if let Some(x) = self.value.as_ref() {
            x
        } else {
            match () {
                // Try to catch UB when compiling in release with debug assertions enabled
                #[cfg(debug_assertions)]
                () => unreachable!(),
                #[cfg(not(debug_assertions))]
                () => hint::unreachable_unchecked(),
            }
        }
    }

    pub unsafe fn get_mut(&mut self) -> &mut T {
        if let Some(x) = self.value.as_mut() {
            x
        } else {
            match () {
                // Try to catch UB when compiling in release with debug assertions enabled
                #[cfg(debug_assertions)]
                () => unreachable!(),
                #[cfg(not(debug_assertions))]
                () => hint::unreachable_unchecked(),
            }
        }
    }

    pub fn set(&mut self, val: T) {
        // NOTE(volatile) we have observed UB when this uses a plain `ptr::write`
        unsafe { ptr::write_volatile(&mut self.value, Some(val)) }
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
    priority: &Cell<u8>,
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
    priority: &Cell<u8>,
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
