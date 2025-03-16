//! An async aware MPSC channel that can be used on no-alloc systems.

#[allow(clippy::module_inception)]
mod channel;
pub use channel::Channel;

mod sender;
pub use sender::{Sender, TrySendError};

mod receiver;
pub use receiver::{ReceiveError, Receiver};

#[doc(hidden)]
pub use critical_section;

/// Creates a split channel with `'static` lifetime.
#[macro_export]
macro_rules! make_channel {
    ($type:ty, $size:expr) => {{
        static mut CHANNEL: $crate::channel::Channel<$type, $size> =
            $crate::channel::Channel::new();

        static CHECK: $crate::portable_atomic::AtomicU8 = $crate::portable_atomic::AtomicU8::new(0);

        $crate::channel::critical_section::with(|_| {
            if CHECK.load(::core::sync::atomic::Ordering::Relaxed) != 0 {
                panic!("call to the same `make_channel` instance twice");
            }

            CHECK.store(1, ::core::sync::atomic::Ordering::Relaxed);
        });

        // SAFETY: This is safe as we hide the static mut from others to access it.
        // Only this point is where the mutable access happens.
        #[allow(static_mut_refs)]
        unsafe {
            CHANNEL.split()
        }
    }};
}
