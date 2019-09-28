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
    init_logic: u8,
    current_logic: Cell<u8>,
    #[cfg(armv7m)]
    old_basepri_hw: Cell<Option<u8>>,
}

impl Priority {
    #[inline(always)]
    pub unsafe fn new(value: u8) -> Self {
        Priority {
            init_logic: value,
            current_logic: Cell::new(value),
            old_basepri_hw: Cell::new(None),
        }
    }

    #[inline(always)]
    fn set_logic(&self, value: u8) {
        self.current_logic.set(value)
    }

    #[inline(always)]
    fn get_logic(&self) -> u8 {
        self.current_logic.get()
    }

    #[inline(always)]
    fn get_init_logic(&self) -> u8 {
        self.init_logic
    }

    #[cfg(armv7m)]
    #[inline(always)]
    fn get_old_basepri_hw(&self) -> Option<u8> {
        self.old_basepri_hw.get()
    }

    #[cfg(armv7m)]
    #[inline(always)]
    fn set_old_basepri_hw(&self, value: u8) {
        self.old_basepri_hw.set(Some(value));
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
    let current = priority.get_logic();

    if current < ceiling {
        if ceiling == (1 << nvic_prio_bits) {
            priority.set_logic(u8::max_value());
            let r = interrupt::free(|_| f(&mut *ptr));
            priority.set_logic(current);
            r
        } else {
            match priority.get_old_basepri_hw() {
                None => priority.set_old_basepri_hw(basepri::read()),
                _ => (),
            };
            priority.set_logic(ceiling);
            basepri::write(logical2hw(ceiling, nvic_prio_bits));
            let r = f(&mut *ptr);
            if current == priority.get_init_logic() {
                basepri::write(priority.get_old_basepri_hw().unwrap());
            } else {
                basepri::write(logical2hw(priority.get_logic(), nvic_prio_bits));
            }
            priority.set_logic(current);
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
        priority.set_logic(u8::max_value());
        let r = interrupt::free(|_| f(&mut *ptr));
        priority.set_logic(current);
        r
    } else {
        f(&mut *ptr)
    }
}

#[inline]
pub fn logical2hw(logical: u8, nvic_prio_bits: u8) -> u8 {
    ((1 << nvic_prio_bits) - logical) << (8 - nvic_prio_bits)
}
