// use riscv::register::{mie, mstatus};

pub use riscv::asm::nop;
pub use riscv_slic;
pub use riscv_slic::swi::InterruptNumber;

#[cfg(feature = "e310x-slic")]
pub use e310x_slic::*;
#[cfg(feature = "e310x-slic")]
pub mod e310x_slic;

/// MANDATORY FOR INTERNAL USE OF MACROS
#[inline(always)]
pub fn run<F>(priority: u8, f: F)
where
    F: FnOnce(),
{
    let old = riscv_slic::get_threshold();
    unsafe {
        riscv_slic::set_threshold(priority);
    }
    f();
    unsafe {
        riscv_slic::set_threshold(old);
    }
}

/// used in bindings macros, we can customize it
#[inline(always)]
pub unsafe fn lock<T, R>(ptr: *mut T, ceiling: u8, f: impl FnOnce(&mut T) -> R) -> R {
    let current = riscv_slic::get_threshold();
    riscv_slic::set_threshold(ceiling);
    let r = f(&mut *ptr);
    riscv_slic::set_threshold(current);
    r
}

/// Sets the given `interrupt` as pending
///
/// MANDATORY FOR INTERNAL USE OF MACROS
pub fn pend<I: riscv_slic::swi::InterruptNumber>(interrupt: I) {
    unsafe {
        riscv_slic::pend(interrupt);
    }
}
