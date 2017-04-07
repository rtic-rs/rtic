//! Safe, run-time checked resources

use core::marker::PhantomData;
use core::cell::UnsafeCell;

use cortex_m::interrupt;
use cortex_m::register::{basepri, basepri_max};
use vcell::VolatileCell;

use Ceiling;

unsafe fn acquire(locked: &VolatileCell<bool>, ceiling: u8) -> u8 {
    interrupt::free(
        |_| {
            assert!(!locked.get(), "resource already locked");
            let old_basepri = basepri::read();
            basepri_max::write(ceiling);
            locked.set(true);
            old_basepri
        },
    )
}

unsafe fn release(locked: &VolatileCell<bool>, old_basepri: u8) {
    // XXX Is it really memory safe to *not* use a global critical section here
    // interrupt::free(
        // |_| {
            locked.set(false);
            basepri::write(old_basepri);
        // },
    // );
}

/// A totally safe `Resource` that panics on misuse
pub struct Resource<T, C> {
    _marker: PhantomData<C>,
    data: UnsafeCell<T>,
    locked: VolatileCell<bool>,
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
            locked: VolatileCell::new(false),
        }
    }

    /// Locks the resource, blocking tasks with priority equal or smaller than
    /// the ceiling `C`
    pub fn lock<R, F>(&'static self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        unsafe {
            let old_basepri = acquire(&self.locked, C::hw_ceiling());
            ::compiler_barrier();
            let ret = f(&*self.data.get());
            ::compiler_barrier();
            release(&self.locked, old_basepri);
            ret
        }
    }

    /// Mutably locks the resource, blocking tasks with priority equal or
    /// smaller than the ceiling `C`
    pub fn lock_mut<R, F>(&'static self, f: F) -> R
        where
        F: FnOnce(&mut T) -> R,
    {
        unsafe {
            let old_basepri = acquire(&self.locked, C::hw_ceiling());
            ::compiler_barrier();
            let ret = f(&mut *self.data.get());
            ::compiler_barrier();
            release(&self.locked, old_basepri);
            ret
        }
    }
}

unsafe impl<T, C> Sync for Resource<T, C> {}
