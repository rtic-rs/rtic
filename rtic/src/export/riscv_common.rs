/// GENERIC RE-EXPORTS: needed for all RTIC backends

/// Read the stack pointer.
#[inline(always)]
pub fn read_sp() -> u32 {
    let r;
    unsafe { core::arch::asm!("mv {}, sp", out(reg) r, options(nomem, nostack, preserves_flags)) };
    r
}
