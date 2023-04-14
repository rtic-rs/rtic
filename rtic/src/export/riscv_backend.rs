/// GENERIC RE-EXPORTS: needed for all RTIC backends
pub use riscv::{asm::nop, interrupt};

#[cfg(all(feature = "riscv-slic", not(any(feature = "riscv-slic-backend"))))]
compile_error!("Building for RISC-V SLIC, but 'riscv-slic-backend' backend not selected");

#[cfg(feature = "riscv-slic")]
pub use slic::*;

#[cfg(feature = "riscv-slic")]
pub mod slic;
