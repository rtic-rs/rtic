//! A "latest only" value store with unlimited writers and async waiting. Value is always available once initialized.

use crate::signal::{Signal, SignalWriter, Store};

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

    /// Returns the latest value, or None if uninitialized.
    pub fn try_get(&mut self) -> Option<T> {
        match self.get_inner() {
            Store::Unset => None,
            Store::Set(value) => Some(value),
        }
    }
}

#[cfg(test)]
#[cfg(not(loom))]
mod tests {
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
}
