//! A "latest only" value store with unlimited writers and async waiting. Value is always available once initialized.

use crate::signal::{Signal, SignalWriter, Store};
use core::{future::poll_fn, task::Poll};
use portable_atomic::Ordering::{Acquire, Release};

/// A "latest only" value store with unlimited writers and async waiting. Value is always available once initialized.
pub struct Watch<T: Copy>(Signal<T>);

impl<T> core::fmt::Debug for Watch<T>
where
    T: core::marker::Copy,
{
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        fmt.write_fmt(format_args!(
            "Watch<{}>{{ .. }}",
            core::any::type_name::<T>()
        ))
    }
}

impl<T: Copy> Default for Watch<T> {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl<T: Copy> Send for Watch<T> {}
unsafe impl<T: Copy> Sync for Watch<T> {}

impl<T: Copy> Watch<T> {
    /// Create a new watch.
    pub const fn new() -> Self {
        Self(Signal::new())
    }

    /// Split the watch into a writer and watch reader.
    pub fn split(&self) -> (WatchWriter<'_, T>, WatchReader<'_, T>) {
        (
            WatchWriter(SignalWriter { parent: &self.0 }),
            WatchReader { parent: &self.0 },
        )
    }
}

/// Creates a split watch with `'static` lifetime.
#[macro_export]
macro_rules! make_watch {
    ( $T:ty ) => {{
        static WATCH: $crate::watch::Watch<$T> = $crate::watch::Watch::new();

        WATCH.split()
    }};
}

/// Facilitates the writing of values to a Watch.
#[derive(Clone)]
pub struct WatchWriter<'a, T: Copy>(SignalWriter<'a, T>);

impl<T> core::fmt::Debug for WatchWriter<'_, T>
where
    T: core::marker::Copy,
{
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        fmt.write_fmt(format_args!(
            "WatchWriter<{}>{{ .. }}",
            core::any::type_name::<T>()
        ))
    }
}

impl<T: Copy> WatchWriter<'_, T> {
    /// Write a value to the Watch.
    pub fn write(&mut self, value: T) {
        self.0.write(value);
    }
}

/// Facilitates the async reading of values from the Watch.
pub struct WatchReader<'a, T: Copy> {
    parent: &'a Signal<T>,
}

impl<T> core::fmt::Debug for WatchReader<'_, T>
where
    T: core::marker::Copy,
{
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        fmt.write_fmt(format_args!(
            "WatchReader<{}>{{ .. }}",
            core::any::type_name::<T>()
        ))
    }
}

impl<T: Copy> WatchReader<'_, T> {
    /// Immediately get the latest value stored in the Signal.
    fn get_inner(&mut self) -> Store<T> {
        critical_section::with(|_| {
            // SAFETY: in a cs: exclusive access
            unsafe { self.parent.store.get().read() }
        })
    }

    /// Immediately get the seen attribute stored in the Signal.
    fn get_seen(&mut self) -> bool {
        self.parent.seen.load(Acquire)
    }

    /// Mark value as seen.
    fn mark_seen(&mut self) {
        self.parent.seen.store(true, Release);
    }

    /// Returns the latest value, or None if uninitialized.
    pub fn try_get(&mut self) -> Option<T> {
        match self.get_inner() {
            Store::Unset => None,
            Store::Set(value) => {
                self.mark_seen();
                Some(value)
            }
        }
    }

    /// Wait for an unseen value.
    ///
    /// If the current value is unseen it will be returned immediately.
    ///
    /// If current value is already seen, it will wait for a new value to be written and then read it.
    pub async fn changed(&mut self) -> T {
        poll_fn(|ctx| {
            self.parent.waker.register(ctx.waker());

            if self.get_seen() {
                return Poll::Pending;
            }

            match self.get_inner() {
                Store::Unset => Poll::Pending,
                Store::Set(value) => {
                    self.mark_seen();
                    Poll::Ready(value)
                }
            }
        })
        .await
    }
}

#[cfg(test)]
#[cfg(not(loom))]
mod tests {
    use super::*;
    use static_cell::StaticCell;

    #[test]
    fn empty() {
        let (_writer, mut reader) = make_watch!(u32);

        assert!(reader.try_get().is_none());
    }

    #[test]
    fn ping_pong() {
        let (mut writer, mut reader) = make_watch!(u32);

        writer.write(0xde);
        assert!(reader.try_get().is_some_and(|value| value == 0xde));
    }

    #[test]
    fn latest() {
        let (mut writer, mut reader) = make_watch!(u32);

        writer.write(0xde);
        writer.write(0xad);
        writer.write(0xbe);
        writer.write(0xef);
        assert!(reader.try_get().is_some_and(|value| value == 0xef));
    }

    #[test]
    fn no_consumption() {
        let (mut writer, mut reader) = make_watch!(u32);

        writer.write(0xaa);
        assert!(reader.try_get().is_some_and(|value| value == 0xaa));
        assert!(reader.try_get().is_some_and(|value| value == 0xaa));
    }

    #[tokio::test]
    async fn pending() {
        let (mut writer, mut reader) = make_watch!(u32);

        writer.write(0xaa);

        assert_eq!(reader.changed().await, 0xaa);
    }

    #[tokio::test]
    async fn changed() {
        static READER: StaticCell<WatchReader<u32>> = StaticCell::new();
        let (mut writer, mut reader) = make_watch!(u32);

        writer.write(0xaa);
        assert_eq!(reader.changed().await, 0xaa);

        let reader = READER.init(reader);
        let handle = tokio::spawn(reader.changed());

        tokio::task::yield_now().await; // encourage tokio executor to poll reader future
        assert!(!handle.is_finished()); // verify reader future did not resolve after poll

        writer.write(0xab);
        assert!(handle.await.is_ok_and(|value| value == 0xab));
    }

    #[tokio::test]
    async fn try_get_marks_seen() {
        static READER: StaticCell<WatchReader<u32>> = StaticCell::new();
        let (mut writer, mut reader) = make_watch!(u32);

        writer.write(0xaa);
        assert!(reader.try_get().is_some_and(|value| value == 0xaa));

        let reader = READER.init(reader);
        let handle = tokio::spawn(reader.changed());

        tokio::task::yield_now().await; // encourage tokio executor to poll reader future
        assert!(!handle.is_finished()); // verify reader future did not resolve after poll
    }
}
