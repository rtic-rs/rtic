pub struct Peripherals;

impl Peripherals {
    pub unsafe fn steal() -> Self {
        Self
    }
}

pub mod interrupt {
    pub fn disable() {}
    pub unsafe fn enable() {}
}

#[inline(always)]
pub fn run<F: FnOnce()>(_priority: u8, f: F) {
    f()
}

#[inline(always)]
pub unsafe fn lock<T, R>(ptr: *mut T, _ceiling: u8, f: impl FnOnce(&mut T) -> R) -> R {
    f(unsafe { &mut *ptr })
}

#[inline(always)]
pub fn pend<I>(_int: I) {}

#[inline(always)]
pub fn unpend<I>(_int: I) {}
