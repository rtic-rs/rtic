//! SPI bus sharing using [`Arbiter`]

use super::Arbiter;
use embedded_hal::digital::OutputPin;
use embedded_hal::spi::SpiBus as BlockingSpiBus;
use embedded_hal_async::{
    delay::DelayNs,
    spi::{ErrorType, Operation, SpiBus as AsyncSpiBus, SpiDevice},
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

impl<BUS, CS, D> ErrorType for ArbiterDevice<'_, BUS, CS, D>
where
    BUS: ErrorType,
    CS: OutputPin,
{
    type Error = DeviceError<BUS::Error, CS::Error>;
}

impl<Word, BUS, CS, D> SpiDevice<Word> for ArbiterDevice<'_, BUS, CS, D>
where
    Word: Copy + 'static,
    BUS: AsyncSpiBus<Word>,
    CS: OutputPin,
    D: DelayNs,
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
                    Operation::DelayNs(ns) => match bus.flush().await {
                        Err(e) => Err(e),
                        Ok(()) => {
                            self.delay.delay_ns(*ns).await;
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

/// [`Arbiter`]-based shared bus implementation.
pub struct BlockingArbiterDevice<'a, BUS, CS, D> {
    bus: &'a Arbiter<BUS>,
    cs: CS,
    delay: D,
}

impl<'a, BUS, CS, D> BlockingArbiterDevice<'a, BUS, CS, D> {
    /// Create a new [`BlockingArbiterDevice`].
    pub fn new(bus: &'a Arbiter<BUS>, cs: CS, delay: D) -> Self {
        Self { bus, cs, delay }
    }

    /// Create an `ArbiterDevice` from an `BlockingArbiterDevice`.
    pub fn into_non_blocking(self) -> ArbiterDevice<'a, BUS, CS, D>
    where
        BUS: AsyncSpiBus,
    {
        ArbiterDevice {
            bus: self.bus,
            cs: self.cs,
            delay: self.delay,
        }
    }
}

impl<'a, BUS, CS, D> ErrorType for BlockingArbiterDevice<'a, BUS, CS, D>
where
    BUS: ErrorType,
    CS: OutputPin,
{
    type Error = DeviceError<BUS::Error, CS::Error>;
}

impl<'a, Word, BUS, CS, D> SpiDevice<Word> for BlockingArbiterDevice<'a, BUS, CS, D>
where
    Word: Copy + 'static,
    BUS: BlockingSpiBus<Word>,
    CS: OutputPin,
    D: DelayNs,
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
                    Operation::Read(buf) => bus.read(buf),
                    Operation::Write(buf) => bus.write(buf),
                    Operation::Transfer(read, write) => bus.transfer(read, write),
                    Operation::TransferInPlace(buf) => bus.transfer_in_place(buf),
                    Operation::DelayNs(ns) => match bus.flush() {
                        Err(e) => Err(e),
                        Ok(()) => {
                            self.delay.delay_ns(*ns).await;
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
        let flush_res = bus.flush();
        let cs_res = self.cs.set_high();

        op_res.map_err(DeviceError::Spi)?;
        flush_res.map_err(DeviceError::Spi)?;
        cs_res.map_err(DeviceError::Cs)?;

        Ok(())
    }
}
