/// USE CASE RE-EXPORTS: needed for SLIC-only
pub use riscv_slic::{self, clear_interrupts, codegen, set_interrupts, set_priority};

/// GENERIC RE-EXPORTS: needed for all RTIC backends
#[cfg(feature = "riscv-e310x-backend")]
pub use e310x::Peripherals; // TODO is this REALLY necessary? Can we move it to macros?
pub use riscv_slic::{lock, pend, run, swi::InterruptNumber};
