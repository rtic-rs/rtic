//! A "latest only" value store with unlimited writers and async waiting.

use core::{cell::UnsafeCell, future::poll_fn, task::Poll};
use rtic_common::waker_registration::CriticalSectionWakerRegistration;

/// Basically an Option but for indicating
/// whether the store has been set or not
#[derive(Clone, Copy)]
enum Store<T> {
    Set(T),
    Unset,
}

/// A "latest only" value store with unlimited writers and async waiting.
pub struct Signal<T: Copy> {
    waker: CriticalSectionWakerRegistration,
    store: UnsafeCell<Store<T>>,
}

impl<T> core::fmt::Debug for Signal<T>
where
    T: core::marker::Copy,
{
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        fmt.write_fmt(format_args!(
            "Signal<{}>{{ .. }}",
            core::any::type_name::<T>()
        ))
    }
}

impl<T: Copy> Default for Signal<T> {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl<T: Copy> Send for Signal<T> {}
unsafe impl<T: Copy> Sync for Signal<T> {}

impl<T: Copy> Signal<T> {
    /// Create a new signal.
    pub const fn new() -> Self {
        Self {
            waker: CriticalSectionWakerRegistration::new(),
            store: UnsafeCell::new(Store::Unset),
        }
    }

    /// Split the signal into a writer and reader.
    pub fn split(&self) -> (SignalWriter<'_, T>, SignalReader<'_, T>) {
        (SignalWriter { parent: self }, SignalReader { parent: self })
    }
}

/// Facilitates the writing of values to a Signal.
#[derive(Clone)]
pub struct SignalWriter<'a, T: Copy> {
    parent: &'a Signal<T>,
}

impl<T> core::fmt::Debug for SignalWriter<'_, T>
where
    T: core::marker::Copy,
{
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        fmt.write_fmt(format_args!(
            "SignalWriter<{}>{{ .. }}",
            core::any::type_name::<T>()
        ))
    }
}

impl<T: Copy> SignalWriter<'_, T> {
    /// Write a raw Store value to the Signal.
    fn write_inner(&mut self, value: Store<T>) {
        critical_section::with(|_| {
            // SAFETY: in a cs: exclusive access
            unsafe { self.parent.store.get().replace(value) };
        });

        self.parent.waker.wake();
    }

    /// Write a value to the Signal.
    pub fn write(&mut self, value: T) {
        self.write_inner(Store::Set(value));
    }

    /// Clear the stored value in the Signal (if any).
    pub fn clear(&mut self) {
        self.write_inner(Store::Unset);
    }
}

/// Facilitates the async reading of values from the Signal.
pub struct SignalReader<'a, T: Copy> {
    parent: &'a Signal<T>,
}

impl<T> core::fmt::Debug for SignalReader<'_, T>
where
    T: core::marker::Copy,
{
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        fmt.write_fmt(format_args!(
            "SignalReader<{}>{{ .. }}",
            core::any::type_name::<T>()
        ))
    }
}

impl<T: Copy> SignalReader<'_, T> {
    /// Immediately read and evict the latest value stored in the Signal.
    fn take(&mut self) -> Store<T> {
        critical_section::with(|_| {
            // SAFETY: in a cs: exclusive access
            unsafe { self.parent.store.get().replace(Store::Unset) }
        })
    }

    /// Returns a pending value if present, or None if no value is available.
    ///
    /// Upon read, the stored value is evicted.
    pub fn try_read(&mut self) -> Option<T> {
        match self.take() {
            Store::Unset => None,
            Store::Set(value) => Some(value),
        }
    }

    /// Wait for a new value to be written and read it.
    ///
    /// If a value is already pending it will be returned immediately.
    ///
    /// Upon read, the stored value is evicted.
    pub async fn wait(&mut self) -> T {
        poll_fn(|ctx| {
            self.parent.waker.register(ctx.waker());
            match self.take() {
                Store::Unset => Poll::Pending,
                Store::Set(value) => Poll::Ready(value),
            }
        })
        .await
    }

    /// Wait for a new value to be written and read it.
    ///
    /// If a value is already pending, it will be evicted and a new
    /// value must be written for the wait to resolve.
    ///
    /// Upon read, the stored value is evicted.
    pub async fn wait_fresh(&mut self) -> T {
        self.take();
        self.wait().await
    }
}

/// Creates a split signal with `'static` lifetime.
#[macro_export]
macro_rules! make_signal {
    ( $T:ty ) => {{
        static SIGNAL: $crate::signal::Signal<$T> = $crate::signal::Signal::new();

        SIGNAL.split()
    }};
}

#[cfg(test)]
#[cfg(not(loom))]
mod tests {
    use super::*;
    use static_cell::StaticCell;

    #[test]
    fn empty() {
        let (_writer, mut reader) = make_signal!(u32);

        assert!(reader.try_read().is_none());
    }

    #[test]
    fn ping_pong() {
        let (mut writer, mut reader) = make_signal!(u32);

        writer.write(0xde);
        assert!(reader.try_read().is_some_and(|value| value == 0xde));
    }

    #[test]
    fn latest() {
        let (mut writer, mut reader) = make_signal!(u32);

        writer.write(0xde);
        writer.write(0xad);
        writer.write(0xbe);
        writer.write(0xef);
        assert!(reader.try_read().is_some_and(|value| value == 0xef));
    }

    #[test]
    fn consumption() {
        let (mut writer, mut reader) = make_signal!(u32);

        writer.write(0xaa);
        assert!(reader.try_read().is_some_and(|value| value == 0xaa));
        assert!(reader.try_read().is_none());
    }

    #[tokio::test]
    async fn pending() {
        let (mut writer, mut reader) = make_signal!(u32);

        writer.write(0xaa);

        assert_eq!(reader.wait().await, 0xaa);
    }

    #[tokio::test]
    async fn waiting() {
        static READER: StaticCell<SignalReader<u32>> = StaticCell::new();
        let (mut writer, reader) = make_signal!(u32);

        writer.write(0xaa);

        let reader = READER.init(reader);
        let handle = tokio::spawn(reader.wait_fresh());

        tokio::task::yield_now().await; // encourage tokio executor to poll reader future
        assert!(!handle.is_finished()); // verify reader future did not resolve after poll

        writer.write(0xab);

        assert!(handle.await.is_ok_and(|value| value == 0xab));
    }
}
