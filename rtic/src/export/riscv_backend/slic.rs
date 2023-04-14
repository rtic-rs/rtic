/// USE CASE EXPORTS: needed for SLIC-only
pub use riscv_slic::{codegen, set_priority};

/// GENERIC EXPORTS: needed for all RTIC backends
#[cfg(feature = "riscv-e310x-backend")]
pub use e310x::Peripherals; // TODO is this REALLY necessary?
pub use riscv_slic::{lock, pend, run, swi::InterruptNumber};

pub mod interrupt {
    pub use riscv_slic::{clear_interrupts as disable, set_interrupts as enable};
}
