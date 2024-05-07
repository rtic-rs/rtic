//! A "latest only" value store with unlimited writers and async waiting.
//!
//! Example usage:
//!
//! ```rust
//! fn init(ctx: init::Context) -> (Shared, Local) {
//!     // mono set up among other things...
//!
//!     let (writer, reader) = make_signal!(u8);
//!
//!     writer_task::spawn(writer);
//!     reader_task::spawn(reader);
//! }
//!
//! #[task]
//! async fn writer_task(_ctx: writer_task::Context, mut writer: SignalWriter<u8>) {
//!     for i in 0..10 {
//!         writer.write(i);
//!         Mono::delay(500.millis()).await;
//!     }
//! }
//!
//! #[task]
//! async fn reader_task(_ctx: reader_task::Context, mut reader: SignalReader<u8>) {
//!     loop {
//!         let value = reader.wait().await;
//!         defmt::info("received value: {}", value);
//!     }
//! }
//! ```

use core::{
    cell::UnsafeCell,
    future::poll_fn,
    sync::atomic::{fence, Ordering},
    task::Poll,
};
use rtic_common::waker_registration::CriticalSectionWakerRegistration;

/// Basically an Option but for indicating
/// whether the store has been set or not
#[derive(Clone, Copy)]
enum Store<T> {
    Set(T),
    Unset,
}

/// An async message passing structure with unlimited writers and one reader.
pub struct Signal<T: Copy> {
    waker: CriticalSectionWakerRegistration,
    store: UnsafeCell<Store<T>>,
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
    pub fn split(&'static self) -> (SignalWriter<T>, SignalReader<T>) {
        (SignalWriter { parent: self }, SignalReader { parent: self })
    }
}

/// Fascilitates the writing of values to a Signal.
#[derive(Clone)]
pub struct SignalWriter<T: 'static + Copy> {
    parent: &'static Signal<T>,
}

impl<T: Copy> SignalWriter<T> {
    /// Write a raw Store value to the Signal.
    fn write_inner(&mut self, value: Store<T>) {
        fence(Ordering::SeqCst);

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

/// Fascilitates the async reading of values from the Signal.
pub struct SignalReader<T: 'static + Copy> {
    parent: &'static Signal<T>,
}

impl<T: Copy> SignalReader<T> {
    /// Immediately read and evict the latest value stored in the Signal.
    fn take(&mut self) -> Store<T> {
        critical_section::with(|_| {
            // SAFETY: in a cs: exclusive access
            unsafe { self.parent.store.get().replace(Store::Unset) }
        })
    }

    /// Wait for a new value to be written and read it.
    ///
    /// If a value is already pending it will be returned immediately.
    pub async fn wait(&mut self) -> T {
        poll_fn(|ctx| match self.take() {
            Store::Unset => {
                self.parent.waker.register(ctx.waker());
                Poll::Pending
            }
            Store::Set(value) => Poll::Ready(value),
        })
        .await
    }

    /// Wait for a new value to be written and read it.
    ///
    /// If a value is already pending, it will be evicted and a new
    /// value must be written for the wait to resolve.
    pub async fn wait_fresh(&mut self) -> T {
        self.take();
        self.wait().await
    }
}

/// Convenience macro for creating a Signal.
#[macro_export]
macro_rules! make_signal {
    ( $T:ty ) => {{
        static SIGNAL: Signal<$T> = Signal::new();

        SIGNAL.split()
    }};
}
