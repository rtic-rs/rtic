//! Safe, run-time checked resources

use core::marker::PhantomData;
use core::cell::UnsafeCell;

use cortex_m::interrupt;
use cortex_m::register::{basepri, basepri_max};

use Ceiling;

unsafe fn acquire(locked: &UnsafeCell<bool>, ceiling: u8) -> u8 {
    interrupt::free(
        |_| {
            assert!(!*locked.get(), "resource already locked");
            let old_basepri = basepri::read();
            basepri_max::write(ceiling);
            *locked.get() = true;
            old_basepri
        },
    )
}

unsafe fn release(locked: &UnsafeCell<bool>, old_basepri: u8) {
    interrupt::free(
        |_| {
            basepri::write(old_basepri);
            *locked.get() = false;
        },
    );
}

/// A totally safe `Resource` that panics on misuse
pub struct Resource<T, C> {
    _marker: PhantomData<C>,
    data: UnsafeCell<T>,
    locked: UnsafeCell<bool>,
}

impl<T, C> Resource<T, C>
where
    C: Ceiling,
{
    /// Creates a new `Resource` with ceiling `C`
    pub const fn new(data: T) -> Resource<T, C> {
        Resource {
            _marker: PhantomData,
            data: UnsafeCell::new(data),
            locked: UnsafeCell::new(false),
        }
    }

    /// Locks the resource, blocking tasks with priority equal or smaller than
    /// the ceiling `C`
    pub fn lock<F>(&'static self, f: F)
    where
        F: FnOnce(&T),
    {
        unsafe {
            let old_basepri = acquire(&self.locked, C::ceiling());
            f(&*self.data.get());
            release(&self.locked, old_basepri);
        }
    }

    /// Mutably locks the resource, blocking tasks with priority equal or
    /// smaller than the ceiling `C`
    pub fn lock_mut<F>(&'static self, f: F)
        where
        F: FnOnce(&mut T),
    {
        unsafe {
            let old_basepri = acquire(&self.locked, C::ceiling());
            f(&mut *self.data.get());
            release(&self.locked, old_basepri);
        }
    }
}
