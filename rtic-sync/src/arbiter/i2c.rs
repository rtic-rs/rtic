//! I2C bus sharing using [`Arbiter`]
//!
//! An Example how to use it in RTIC application:
//! ```text
//! #[app(device = some_hal, peripherals = true, dispatchers = [TIM16])]
//! mod app {
//!     use core::mem::MaybeUninit;
//!     use rtic_sync::{arbiter::{i2c::ArbiterDevice, Arbiter},
//!
//!     #[shared]
//!     struct Shared {}
//!
//!     #[local]
//!     struct Local {
//!         ens160: Ens160<ArbiterDevice<'static, I2c<'static, I2C1>>>,
//!     }
//!
//!     #[init(local = [
//!         i2c_arbiter: MaybeUninit<Arbiter<I2c<'static, I2C1>>> = MaybeUninit::uninit(),
//!     ])]
//!     fn init(cx: init::Context) -> (Shared, Local) {
//!         let i2c = I2c::new(cx.device.I2C1);
//!         let i2c_arbiter = cx.local.i2c_arbiter.write(Arbiter::new(i2c));
//!         let ens160 = Ens160::new(ArbiterDevice::new(i2c_arbiter), 0x52);
//!
//!         i2c_sensors::spawn(i2c_arbiter).ok();
//!
//!         (Shared {}, Local { ens160 })
//!     }
//!
//!     #[task(local = [ens160])]
//!     async fn i2c_sensors(cx: i2c_sensors::Context, i2c: &'static Arbiter<I2c<'static, I2C1>>) {
//!         use sensor::Asensor;
//!
//!         loop {
//!             // Use scope to make sure I2C access is dropped.
//!             {
//!                 // Read from sensor driver that wants to use I2C directly.
//!                 let mut i2c = i2c.access().await;
//!                 let status = Asensor::status(&mut i2c).await;
//!             }
//!
//!             // Read ENS160 sensor.
//!             let eco2 = cx.local.ens160.eco2().await;
//!         }
//!     }
//! }
//! ```

use super::Arbiter;
use embedded_hal::i2c::{AddressMode, ErrorType, I2c as BlockingI2c, Operation};
use embedded_hal_async::i2c::I2c as AsyncI2c;

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

impl<BUS> ErrorType for ArbiterDevice<'_, BUS>
where
    BUS: ErrorType,
{
    type Error = BUS::Error;
}

impl<BUS, A> AsyncI2c<A> for ArbiterDevice<'_, BUS>
where
    BUS: AsyncI2c<A>,
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

/// [`Arbiter`]-based shared bus implementation for I2C.
pub struct BlockingArbiterDevice<'a, BUS> {
    bus: &'a Arbiter<BUS>,
}

impl<'a, BUS> BlockingArbiterDevice<'a, BUS> {
    /// Create a new [`BlockingArbiterDevice`] for I2C.
    pub fn new(bus: &'a Arbiter<BUS>) -> Self {
        Self { bus }
    }

    /// Create an `ArbiterDevice` from an `BlockingArbiterDevice`.
    pub fn into_non_blocking(self) -> ArbiterDevice<'a, BUS>
    where
        BUS: AsyncI2c,
    {
        ArbiterDevice { bus: self.bus }
    }
}

impl<'a, BUS> ErrorType for BlockingArbiterDevice<'a, BUS>
where
    BUS: ErrorType,
{
    type Error = BUS::Error;
}

impl<'a, BUS, A> AsyncI2c<A> for BlockingArbiterDevice<'a, BUS>
where
    BUS: BlockingI2c<A>,
    A: AddressMode,
{
    async fn read(&mut self, address: A, read: &mut [u8]) -> Result<(), Self::Error> {
        let mut bus = self.bus.access().await;
        bus.read(address, read)
    }

    async fn write(&mut self, address: A, write: &[u8]) -> Result<(), Self::Error> {
        let mut bus = self.bus.access().await;
        bus.write(address, write)
    }

    async fn write_read(
        &mut self,
        address: A,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        let mut bus = self.bus.access().await;
        bus.write_read(address, write, read)
    }

    async fn transaction(
        &mut self,
        address: A,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        let mut bus = self.bus.access().await;
        bus.transaction(address, operations)
    }
}
