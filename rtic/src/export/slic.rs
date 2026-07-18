pub use riscv_slic::{InterruptNumber, lock, pend, run};

/// USE CASE RE-EXPORTS: needed for SLIC-only
pub use riscv_slic::{self, codegen, set_priority};

pub mod interrupt {
    #[inline]
    pub fn disable() {
        riscv_slic::disable();
    }

    #[inline]
    pub unsafe fn enable() {
        unsafe { riscv_slic::enable() };
    }
}
