/// An [`core::cell::UnsafeCell`] wrapper that provides compatibility with
/// loom's UnsafeCell.
#[derive(Debug)]
pub struct UnsafeCell<T>(core::cell::UnsafeCell<T>);

impl<T> UnsafeCell<T> {
    pub const fn new(data: T) -> UnsafeCell<T> {
        UnsafeCell(core::cell::UnsafeCell::new(data))
    }

    pub const fn with<R>(&self, f: impl FnOnce(*const T) -> R) -> R {
        f(self.0.get())
    }

    pub const fn with_mut<R>(&self, f: impl FnOnce(*mut T) -> R) -> R {
        f(self.0.get())
    }
}
