//! Stack Resource Policy for Cortex-M processors
//!
//! NOTE ARMv6-M is not supported at the moment.

#![feature(asm)]
#![feature(const_fn)]
#![no_std]

// NOTE Only the 4 highest bits of the priority byte (BASEPRI / NVIC.IPR) are
// considered when determining priorities.
const PRIORITY_BITS: u8 = 4;

extern crate cortex_m;

use cortex_m::interrupt::CsToken;
use cortex_m::register::{basepri, basepri_max};

use core::cell::UnsafeCell;
use core::marker::PhantomData;

// XXX why is this needed?
#[inline(always)]
fn compiler_barrier() {
    unsafe {
        asm!(""
             :
             :
             : "memory"
             : "volatile")
    }
}

// XXX why is this needed?
#[inline(always)]
fn memory_barrier() {
    unsafe {
        asm!("dsb
              isb"
             :
             :
             : "memory"
             : "volatile")
    }
}

pub struct Peripheral<T>
    where T: 'static
{
    _ty: PhantomData<&'static mut T>,
    address: usize,
    ceiling: u8,
}

impl<T> Peripheral<T> {
    pub const unsafe fn new(address: usize, ceiling: u8) -> Self {
        Peripheral {
            _ty: PhantomData,
            address: address,
            ceiling: ceiling,
        }
    }

    pub fn claim<F, R>(&self, f: F) -> R
        where F: FnOnce(&T) -> R
    {
        unsafe {
            let old_basepri = basepri::read();
            basepri_max::write(priority(self.ceiling));
            memory_barrier();
            let ret = f(&*(self.address as *const T));
            compiler_barrier();
            basepri::write(old_basepri);
            ret
        }
    }

    pub fn claim_mut<F, R>(&self, f: F) -> R
        where F: FnOnce(&mut T) -> R
    {
        unsafe {
            let old_basepri = basepri::read();
            basepri_max::write(priority(self.ceiling));
            memory_barrier();
            let ret = f(&mut *(self.address as *mut T));
            compiler_barrier();
            basepri::write(old_basepri);
            ret
        }
    }

    pub fn take<'a>(&self, _token: &'a CsToken) -> &'a T {
        unsafe {
            &*(self.address as *const T)
        }
    }

    pub fn take_mut<'a>(&self, _token: &'a CsToken) -> &'a mut T {
        unsafe {
            &mut *(self.address as *mut T)
        }
    }
}

pub struct Resource<T> {
    ceiling: u8,
    data: UnsafeCell<T>,
}

impl<T> Resource<T> {
    pub const fn new(data: T, ceiling: u8) -> Self {
        Resource {
            ceiling: ceiling,
            data: UnsafeCell::new(data),
        }
    }

    pub fn claim<F, R>(&self, f: F) -> R
        where F: FnOnce(&T) -> R
    {
        unsafe {
            let old_basepri = basepri::read();
            basepri_max::write(priority(self.ceiling));
            memory_barrier();
            let ret = f(&*self.data.get());
            compiler_barrier();
            basepri::write(old_basepri);
            ret
        }
    }

    pub fn claim_mut<F, R>(&self, f: F) -> R
        where F: FnOnce(&mut T) -> R
    {
        unsafe {
            let old_basepri = basepri::read();
            basepri_max::write(priority(self.ceiling));
            memory_barrier();
            let ret = f(&mut *self.data.get());
            compiler_barrier();
            basepri::write(old_basepri);
            ret
        }
    }

    pub fn take<'a>(&self, _token: &'a CsToken) -> &'a T {
        unsafe {
            &*self.data.get()
        }
    }

    pub fn take_mut<'a>(&self, _token: &'a CsToken) -> &'a mut T {
        unsafe {
            &mut *self.data.get()
        }
    }
}

unsafe impl<T> Sync for Resource<T> {}

/// Turns a `logical` priority into a NVIC-style priority
///
/// With `logical` priorities, `2` has HIGHER priority than `1`.
///
/// With NVIC priorities, `32` has LOWER priority than `16`. (Also, NVIC
/// priorities encode the actual priority in the highest bits of a byte so
/// priorities like `1` and `2` aren't actually different)
// TODO review the handling of extreme values
pub const fn priority(logical: u8) -> u8 {
    ((1 << PRIORITY_BITS) - logical) << (8 - PRIORITY_BITS)
}
