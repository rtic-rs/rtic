pub use riscv_slic::{lock, pend, run, InterruptNumber};

#[cfg(all(feature = "riscv-slic", not(feature = "riscv-clint-backend")))]
compile_error!("Building for the riscv-slic, but 'riscv-clint-backend' not selected");

pub struct Peripherals(); // TODO remove Peripherals from RTIC

/// USE CASE RE-EXPORTS: needed for SLIC-only
pub use riscv_slic::{self, clear_interrupts, codegen, set_interrupts, set_priority};
