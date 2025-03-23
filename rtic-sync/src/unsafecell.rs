//! Compat layer for [`core::cell::UnsafeCell`] and `loom::cell::UnsafeCell`.

#[cfg(loom)]
pub use loom::cell::UnsafeCell;

#[cfg(not(loom))]
pub use core::UnsafeCell;

#[cfg(not(loom))]
mod core {
    /// An [`core::cell::UnsafeCell`] wrapper that provides compatibility with
    /// loom's UnsafeCell.
    #[derive(Debug)]
    pub struct UnsafeCell<T>(core::cell::UnsafeCell<T>);

    impl<T> UnsafeCell<T> {
        /// Create a new `UnsafeCell`.
        pub const fn new(data: T) -> UnsafeCell<T> {
            UnsafeCell(core::cell::UnsafeCell::new(data))
        }

        /// Access the contents of the `UnsafeCell` through a mut pointer.
        pub fn get_mut(&self) -> MutPtr<T> {
            MutPtr(self.0.get())
        }
    }

    pub struct MutPtr<T>(*mut T);

    impl<T> MutPtr<T> {
        #[allow(clippy::mut_from_ref)]
        /// SAFETY: the caller must guarantee that the contained `*mut T` is not
        /// null, and must uphold the same safety requirements as for
        /// [`core::primitive::pointer::as_mut`] for the contained `*mut T`.
        pub unsafe fn deref(&self) -> &mut T {
            &mut *self.0
        }
    }
}
