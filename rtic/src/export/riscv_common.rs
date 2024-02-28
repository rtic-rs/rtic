/// GENERIC RE-EXPORTS: needed for all RTIC backends
pub use riscv::interrupt;

/// Read the stack pointer.
#[inline(always)]
pub fn read_sp() -> u32 {
    let r;
    unsafe { asm!("mv {}, sp", out(reg) r, options(nomem, nostack, preserves_flags)) };
    r
}
