use riscv::riscv::register::{mie, mstatus};

pub use e310x::Peripherals;

pub use riscv::{
    asm::nop,                                           // MANDATORY FOR INTERNAL USE OF MACROS
    peripheral::plic::{InterruptNumber, PriorityLevel}, // Generic PLIC-related traits
};
pub use riscv_vsoft;
pub use riscv_vsoft_macros;

/// Codegen code for the software interrupt controller
riscv_vsoft_macros::codegen!(10);

pub mod interrupt {

    /// MANDATORY FOR INTERNAL USE OF MACROS
    #[inline(always)]
    pub fn disable() {
        riscv_vsoft::common::disable();
    }

    /// MANDATORY FOR INTERNAL USE OF MACROS
    #[inline(always)]
    pub fn enable() {
        riscv_vsoft::common::enable();
        // __SOFTWARE_CONTROLLER.set_threshold(0);
    }

    /// used in bindings macros, we can customize it
    #[inline(always)]
    pub fn source_priority(interrupt: Interrupt, priority: u8) {
        let priority = Priority::try_from(priority).unwrap();
        __SOFTWARE_CONTROLLER.set_priority(interrupt as u16, priority);
    }
}

/// MANDATORY FOR INTERNAL USE OF MACROS
#[inline(always)]
pub fn run<F>(priority: u8, f: F)
where
    F: FnOnce(),
{
    let priority = Priority::try_from(priority as u8).unwrap();
    let current: Priority = __SOFTWARE_CONTROLLER.get_threshold();
    __SOFTWARE_CONTROLLER.set_threshold(priority);
    f();
    __SOFTWARE_CONTROLLER.set_threshold(current);
}

/// used in bindings macros, we can customize it
#[inline(always)]
pub unsafe fn lock<T, R>(ptr: *mut T, ceiling: u8, f: impl FnOnce(&mut T) -> R) -> R {
    let ceiling = Priority::try_from(ceiling).unwrap();
    let current: Priority = __SOFTWARE_CONTROLLER.get_threshold();
    __SOFTWARE_CONTROLLER.set_threshold(ceiling);
    let r = f(&mut *ptr);
    __SOFTWARE_CONTROLLER.set_threshold(current);
    r
}

/// Sets the given `interrupt` as pending
///
/// MANDATORY FOR INTERNAL USE OF MACROS
pub fn pend<I: InterruptNumber>(interrupt: I) {
    __SOFTWARE_CONTROLLER.pend(interrupt.into());
}
