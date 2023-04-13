pub use riscv_slic::{self, lock, pend, riscv::asm::nop, run, swi::InterruptNumber};

#[cfg(feature = "riscv-e310x-backend")]
pub use e310x::Peripherals;
