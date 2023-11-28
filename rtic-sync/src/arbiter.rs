//! A Mutex-like FIFO with unlimited-waiter for embedded systems.
//!
//! Example usage:
//!
//! ```rust
//! # async fn select<F1, F2>(f1: F1, f2: F2) {}
//! use rtic_sync::arbiter::Arbiter;
//!
//! // Instantiate an Arbiter with a static lifetime.
//! static ARBITER: Arbiter<u32> = Arbiter::new(32);
//!
//! async fn run(){
//!     let write_42 = async move {
//!         *ARBITER.access().await = 42;
//!     };
//!
//!     let write_1337 = async move {
//!         *ARBITER.access().await = 1337;
//!     };
//!
//!     // Attempt to access the Arbiter concurrently.
//!     select(write_42, write_1337).await;
//! }
//! ```

use core::cell::UnsafeCell;
use core::future::poll_fn;
use core::ops::{Deref, DerefMut};
use core::pin::Pin;
use core::task::{Poll, Waker};
use portable_atomic::{fence, AtomicBool, Ordering};

use rtic_common::dropper::OnDrop;
use rtic_common::wait_queue::{Link, WaitQueue};

/// This is needed to make the async closure in `send` accept that we "share"
/// the link possible between threads.
#[derive(Clone)]
struct LinkPtr(*mut Option<Link<Waker>>);

impl LinkPtr {
    /// This will dereference the pointer stored within and give out an `&mut`.
    unsafe fn get(&mut self) -> &mut Option<Link<Waker>> {
        &mut *self.0
    }
}

unsafe impl Send for LinkPtr {}
unsafe impl Sync for LinkPtr {}

/// An FIFO waitqueue for use in shared bus usecases.
pub struct Arbiter<T> {
    wait_queue: WaitQueue,
    inner: UnsafeCell<T>,
    taken: AtomicBool,
}

unsafe impl<T> Send for Arbiter<T> {}
unsafe impl<T> Sync for Arbiter<T> {}

impl<T> Arbiter<T> {
    /// Create a new arbiter.
    pub const fn new(inner: T) -> Self {
        Self {
            wait_queue: WaitQueue::new(),
            inner: UnsafeCell::new(inner),
            taken: AtomicBool::new(false),
        }
    }

    /// Get access to the inner value in the [`Arbiter`]. This will wait until access is granted,
    /// for non-blocking access use `try_access`.
    pub async fn access(&self) -> ExclusiveAccess<'_, T> {
        let mut link_ptr: Option<Link<Waker>> = None;

        // Make this future `Drop`-safe.
        // SAFETY(link_ptr): Shadow the original definition of `link_ptr` so we can't abuse it.
        let mut link_ptr = LinkPtr(&mut link_ptr as *mut Option<Link<Waker>>);

        let mut link_ptr2 = link_ptr.clone();
        let dropper = OnDrop::new(|| {
            // SAFETY: We only run this closure and dereference the pointer if we have
            // exited the `poll_fn` below in the `drop(dropper)` call. The other dereference
            // of this pointer is in the `poll_fn`.
            if let Some(link) = unsafe { link_ptr2.get() } {
                link.remove_from_list(&self.wait_queue);
            }
        });

        poll_fn(|cx| {
            critical_section::with(|_| {
                fence(Ordering::SeqCst);

                // The queue is empty and noone has taken the value.
                if self.wait_queue.is_empty() && !self.taken.load(Ordering::Relaxed) {
                    self.taken.store(true, Ordering::Relaxed);

                    return Poll::Ready(());
                }

                // SAFETY: This pointer is only dereferenced here and on drop of the future
                // which happens outside this `poll_fn`'s stack frame.
                let link = unsafe { link_ptr.get() };
                if let Some(link) = link {
                    if link.is_popped() {
                        return Poll::Ready(());
                    }
                } else {
                    // Place the link in the wait queue on first run.
                    let link_ref = link.insert(Link::new(cx.waker().clone()));

                    // SAFETY(new_unchecked): The address to the link is stable as it is defined
                    // outside this stack frame.
                    // SAFETY(push): `link_ref` lifetime comes from `link_ptr` that is shadowed,
                    // and  we make sure in `dropper` that the link is removed from the queue
                    // before dropping `link_ptr` AND `dropper` makes sure that the shadowed
                    // `link_ptr` lives until the end of the stack frame.
                    unsafe { self.wait_queue.push(Pin::new_unchecked(link_ref)) };
                }

                Poll::Pending
            })
        })
        .await;

        // Make sure the link is removed from the queue.
        drop(dropper);

        // SAFETY: One only gets here if there is exlusive access.
        ExclusiveAccess {
            arbiter: self,
            inner: unsafe { &mut *self.inner.get() },
        }
    }

    /// Non-blockingly tries to access the underlying value.
    /// If someone is in queue to get it, this will return `None`.
    pub fn try_access(&self) -> Option<ExclusiveAccess<'_, T>> {
        critical_section::with(|_| {
            fence(Ordering::SeqCst);

            // The queue is empty and noone has taken the value.
            if self.wait_queue.is_empty() && !self.taken.load(Ordering::Relaxed) {
                self.taken.store(true, Ordering::Relaxed);

                // SAFETY: One only gets here if there is exlusive access.
                Some(ExclusiveAccess {
                    arbiter: self,
                    inner: unsafe { &mut *self.inner.get() },
                })
            } else {
                None
            }
        })
    }
}

/// This token represents exclusive access to the value protected by the [`Arbiter`].
pub struct ExclusiveAccess<'a, T> {
    arbiter: &'a Arbiter<T>,
    inner: &'a mut T,
}

impl<'a, T> Drop for ExclusiveAccess<'a, T> {
    fn drop(&mut self) {
        critical_section::with(|_| {
            fence(Ordering::SeqCst);

            if self.arbiter.wait_queue.is_empty() {
                // If noone is in queue and we release exclusive access, reset `taken`.
                self.arbiter.taken.store(false, Ordering::Relaxed);
            } else if let Some(next) = self.arbiter.wait_queue.pop() {
                // Wake the next one in queue.
                next.wake();
            }
        })
    }
}

impl<'a, T> Deref for ExclusiveAccess<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, T> DerefMut for ExclusiveAccess<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}

#[cfg(feature = "unstable")]
/// SPI bus sharing using [`Arbiter`]
pub mod spi {
    use super::Arbiter;
    use embedded_hal::digital::OutputPin;
    use embedded_hal_async::{
        delay::DelayUs,
        spi::{ErrorType, Operation, SpiBus, SpiDevice},
    };
    use embedded_hal_bus::spi::DeviceError;

    /// [`Arbiter`]-based shared bus implementation.
    pub struct ArbiterDevice<'a, BUS, CS, D> {
        bus: &'a Arbiter<BUS>,
        cs: CS,
        delay: D,
    }

    impl<'a, BUS, CS, D> ArbiterDevice<'a, BUS, CS, D> {
        /// Create a new [`ArbiterDevice`].
        pub fn new(bus: &'a Arbiter<BUS>, cs: CS, delay: D) -> Self {
            Self { bus, cs, delay }
        }
    }

    impl<'a, BUS, CS, D> ErrorType for ArbiterDevice<'a, BUS, CS, D>
    where
        BUS: ErrorType,
        CS: OutputPin,
    {
        type Error = DeviceError<BUS::Error, CS::Error>;
    }

    impl<'a, Word, BUS, CS, D> SpiDevice<Word> for ArbiterDevice<'a, BUS, CS, D>
    where
        Word: Copy + 'static,
        BUS: SpiBus<Word>,
        CS: OutputPin,
        D: DelayUs,
    {
        async fn transaction(
            &mut self,
            operations: &mut [Operation<'_, Word>],
        ) -> Result<(), DeviceError<BUS::Error, CS::Error>> {
            let mut bus = self.bus.access().await;

            self.cs.set_low().map_err(DeviceError::Cs)?;

            let op_res = 'ops: {
                for op in operations {
                    let res = match op {
                        Operation::Read(buf) => bus.read(buf).await,
                        Operation::Write(buf) => bus.write(buf).await,
                        Operation::Transfer(read, write) => bus.transfer(read, write).await,
                        Operation::TransferInPlace(buf) => bus.transfer_in_place(buf).await,
                        Operation::DelayUs(us) => match bus.flush().await {
                            Err(e) => Err(e),
                            Ok(()) => {
                                self.delay.delay_us(*us).await;
                                Ok(())
                            }
                        },
                    };
                    if let Err(e) = res {
                        break 'ops Err(e);
                    }
                }
                Ok(())
            };

            // On failure, it's important to still flush and deassert CS.
            let flush_res = bus.flush().await;
            let cs_res = self.cs.set_high();

            op_res.map_err(DeviceError::Spi)?;
            flush_res.map_err(DeviceError::Spi)?;
            cs_res.map_err(DeviceError::Cs)?;

            Ok(())
        }
    }
}

#[cfg(feature = "unstable")]
/// I2C bus sharing using [`Arbiter`]
///
/// An Example how to use it in RTIC application:
/// ```ignore
/// #[app(device = some_hal, peripherals = true, dispatchers = [TIM16])]
/// mod app {
///     use core::mem::MaybeUninit;
///     use rtic_sync::{arbiter::{i2c::ArbiterDevice, Arbiter},
///
///     #[shared]
///     struct Shared {}
///
///     #[local]
///     struct Local {
///         ens160: Ens160<ArbiterDevice<'static, I2c<'static, I2C1>>>,
///     }
///
///     #[init(local = [
///         i2c_arbiter: MaybeUninit<Arbiter<I2c<'static, I2C1>>> = MaybeUninit::uninit(),
///     ])]
///     fn init(cx: init::Context) -> (Shared, Local) {
///         let i2c = I2c::new(cx.device.I2C1);
///         let i2c_arbiter = cx.local.i2c_arbiter.write(Arbiter::new(i2c));
///         let ens160 = Ens160::new(ArbiterDevice::new(i2c_arbiter), 0x52);
///
///         i2c_sensors::spawn(i2c_arbiter).ok();
///
///         (Shared {}, Local { ens160 })
///     }
///
///     #[task(local = [ens160])]
///     async fn i2c_sensors(cx: i2c_sensors::Context, i2c: &'static Arbiter<I2c<'static, I2C1>>) {
///         use sensor::Asensor;
///
///         loop {
///             // Use scope to make sure I2C access is dropped.
///             {
///                 // Read from sensor driver that wants to use I2C directly.
///                 let mut i2c = i2c.access().await;
///                 let status = Asensor::status(&mut i2c).await;
///             }
///
///             // Read ENS160 sensor.
///             let eco2 = cx.local.ens160.eco2().await;
///         }
///     }
/// }
/// ```
pub mod i2c {
    use super::Arbiter;
    use embedded_hal::i2c::{AddressMode, ErrorType, Operation};
    use embedded_hal_async::i2c::I2c;

    /// [`Arbiter`]-based shared bus implementation for I2C.
    pub struct ArbiterDevice<'a, BUS> {
        bus: &'a Arbiter<BUS>,
    }

    impl<'a, BUS> ArbiterDevice<'a, BUS> {
        /// Create a new [`ArbiterDevice`] for I2C.
        pub fn new(bus: &'a Arbiter<BUS>) -> Self {
            Self { bus }
        }
    }

    impl<'a, BUS> ErrorType for ArbiterDevice<'a, BUS>
    where
        BUS: ErrorType,
    {
        type Error = BUS::Error;
    }

    impl<'a, BUS, A> I2c<A> for ArbiterDevice<'a, BUS>
    where
        BUS: I2c<A>,
        A: AddressMode,
    {
        async fn read(&mut self, address: A, read: &mut [u8]) -> Result<(), Self::Error> {
            let mut bus = self.bus.access().await;
            bus.read(address, read).await
        }

        async fn write(&mut self, address: A, write: &[u8]) -> Result<(), Self::Error> {
            let mut bus = self.bus.access().await;
            bus.write(address, write).await
        }

        async fn write_read(
            &mut self,
            address: A,
            write: &[u8],
            read: &mut [u8],
        ) -> Result<(), Self::Error> {
            let mut bus = self.bus.access().await;
            bus.write_read(address, write, read).await
        }

        async fn transaction(
            &mut self,
            address: A,
            operations: &mut [Operation<'_>],
        ) -> Result<(), Self::Error> {
            let mut bus = self.bus.access().await;
            bus.transaction(address, operations).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn stress_channel() {
        const NUM_RUNS: usize = 100_000;

        static ARB: Arbiter<usize> = Arbiter::new(0);
        let mut v = std::vec::Vec::new();

        for _ in 0..NUM_RUNS {
            v.push(tokio::spawn(async move {
                *ARB.access().await += 1;
            }));
        }

        for v in v {
            v.await.unwrap();
        }

        assert_eq!(*ARB.access().await, NUM_RUNS)
    }
}
