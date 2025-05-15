//! A drop implementation runner.

use core::ops::{Deref, DerefMut};

pub(crate) struct OnDropWith<T, F: FnMut(&mut T)>(T, F);

/// Runs a closure on drop.
pub struct OnDrop<F: FnOnce()> {
    f: core::mem::MaybeUninit<F>,
}

impl<F: FnOnce()> OnDrop<F> {
    /// Make a new droppper given a closure.
    pub fn new(f: F) -> Self {
        Self {
            f: core::mem::MaybeUninit::new(f),
        }
    }

    /// Make it not run drop.
    pub fn defuse(self) {
        core::mem::forget(self)
    }
}

impl<F: FnOnce()> Drop for OnDrop<F> {
    fn drop(&mut self) {
        unsafe { self.f.as_ptr().read()() }
    }
}

impl<T, F: FnMut(&mut T)> OnDropWith<T, F> {
    pub(crate) fn new(value: T, f: F) -> Self {
        Self(value, f)
    }

    pub(crate) fn execute(&mut self) {
        (self.1)(&mut self.0);
    }
}

impl<T, F: FnMut(&mut T)> Deref for OnDropWith<T, F> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, F: FnMut(&mut T)> DerefMut for OnDropWith<T, F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T, F: FnMut(&mut T)> Drop for OnDropWith<T, F> {
    fn drop(&mut self) {
        self.execute();
    }
}
