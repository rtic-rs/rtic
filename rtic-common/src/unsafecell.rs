//! Loom-compatible [`core::cell::UnsafeCell`].

/// An [`core::cell::UnsafeCell`] wrapper that provides compatibility with
/// loom's UnsafeCell.
#[derive(Debug)]
pub struct UnsafeCell<T>(core::cell::UnsafeCell<T>);

impl<T> UnsafeCell<T> {
    /// Create a new `UnsafeCell`.
    pub const fn new(data: T) -> UnsafeCell<T> {
        UnsafeCell(core::cell::UnsafeCell::new(data))
    }

    /// Access the contents of the `UnsafeCell` through a const pointer.
    pub fn with<R>(&self, f: impl FnOnce(*const T) -> R) -> R {
        f(self.0.get())
    }

    /// Access the contents of the `UnsafeCell` through a mut pointer.
    pub fn with_mut<R>(&self, f: impl FnOnce(*mut T) -> R) -> R {
        f(self.0.get())
    }

    /// Consume the UnsafeCell.
    pub fn into_inner(self) -> T {
        self.0.into_inner()
    }
}
