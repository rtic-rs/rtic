pub use riscv_slic::{lock, pend, run, InterruptNumber};

#[cfg(all(feature = "riscv-slic", not(feature = "riscv-clint-backend")))]
compile_error!("Building for the riscv-slic, but 'riscv-clint-backend' not selected");

/// USE CASE RE-EXPORTS: needed for SLIC-only
pub use riscv_slic::{self, clear_interrupts, codegen, set_interrupts, set_priority};

pub mod interrupt {
    pub fn disable() {
        riscv_slic::disable();
        riscv_slic::clear_interrupts();
    }

    pub unsafe fn enable() {
        riscv_slic::set_interrupts();
        riscv_slic::enable();
    }
}
