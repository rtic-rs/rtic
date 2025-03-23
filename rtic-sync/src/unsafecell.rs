//! Compat layer for [`core::cell::UnsafeCell`] and `loom::cell::UnsafeCell`.

#[cfg(loom)]
use loom::cell::UnsafeCell as InnerUnsafeCell;

#[cfg(loom)]
pub use loom::cell::MutPtr;

#[cfg(not(loom))]
use core::cell::UnsafeCell as InnerUnsafeCell;

/// An [`core::cell::UnsafeCell`] wrapper that provides compatibility with
/// loom's UnsafeCell.
#[derive(Debug)]
pub struct UnsafeCell<T>(InnerUnsafeCell<T>);

impl<T> UnsafeCell<T> {
    /// Create a new `UnsafeCell`.
    #[cfg(not(loom))]
    pub const fn new(data: T) -> UnsafeCell<T> {
        UnsafeCell(InnerUnsafeCell::new(data))
    }

    #[cfg(loom)]
    pub fn new(data: T) -> UnsafeCell<T> {
        UnsafeCell(InnerUnsafeCell::new(data))
    }

    /// Access the contents of the `UnsafeCell` through a tracked mut pointer.
    pub fn get_mut(&self) -> MutPtr<T> {
        #[cfg(loom)]
        return self.0.get_mut();

        #[cfg(not(loom))]
        return MutPtr(self.0.get());
    }

    /// Access the contents of the `UnsafeCell` mutably.
    pub fn as_mut(&mut self) -> &mut T {
        #[cfg(not(loom))]
        return self.0.get_mut();

        #[cfg(loom)]
        {
            // SAFETY: we have exclusive access to `self`.
            let ptr = self.get_mut();
            let ptr = unsafe { ptr.deref() };

            // SAFETY: we have exclusive access to `self` for the duration of
            // the borrow.
            unsafe { core::mem::transmute(ptr) }
        }
    }
}

#[cfg(not(loom))]
pub struct MutPtr<T>(*mut T);

#[cfg(not(loom))]
impl<T> MutPtr<T> {
    #[allow(clippy::mut_from_ref)]
    /// SAFETY: the caller must guarantee that the contained `*mut T` is not
    /// null, and must uphold the same safety requirements as for
    /// [`core::primitive::pointer::as_mut`] for the contained `*mut T`.
    pub unsafe fn deref(&self) -> &mut T {
        &mut *self.0
    }
}
