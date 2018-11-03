/// IMPLEMENTATION DETAILS. DO NOT USE ANYTHING IN THIS MODULE
use core::{hint, ptr};

#[cfg(armv7m)]
use cortex_m::register::basepri;
pub use cortex_m::{
    asm::wfi, interrupt, peripheral::scb::SystemHandler, peripheral::syst::SystClkSource,
    peripheral::Peripherals,
};
pub use cortex_m_rt::{entry, exception};
pub use heapless::consts;
use heapless::spsc::Queue;

#[cfg(feature = "timer-queue")]
pub use crate::tq::{isr as sys_tick, NotReady, TimerQueue};

pub type FreeQueue<N> = Queue<u8, N>;
pub type ReadyQueue<T, N> = Queue<(T, u8), N>;

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
            hint::unreachable_unchecked()
        }
    }

    pub unsafe fn get_mut(&mut self) -> &mut T {
        if let Some(x) = self.value.as_mut() {
            x
        } else {
            hint::unreachable_unchecked()
        }
    }

    pub fn set(&mut self, val: T) {
        unsafe { ptr::write(&mut self.value, Some(val)) }
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
