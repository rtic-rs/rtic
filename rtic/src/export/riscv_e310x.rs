use riscv::riscv::register::{mie, mstatus};

pub use e310x::{
    plic::{Interrupt, Priority}, // target-specific PLIC-related enums
    Peripherals,                 // MANDATORY FOR INTERNAL USE OF MACROS
    PLIC0,                       // Contains a PLIC field
};
pub use riscv::{
    asm::nop,                                           // MANDATORY FOR INTERNAL USE OF MACROS
    peripheral::plic::{InterruptNumber, PriorityLevel}, // Generic PLIC-related traits
};

pub mod interrupt {

    /// MANDATORY FOR INTERNAL USE OF MACROS
    #[inline(always)]
    pub fn disable() {
        mstatus::clear_mie();
        mie::clear_mext();
        reset_plic();
    }

    /// used in bindings macros, we can customize it
    #[inline(always)]
    pub fn reset_plic() {
        let plic = Peripherals::steal().PLIC;
        for i in 1..=Interrupt::MAX_INTERRUPT_NUMBER {
            let interrupt = Interrupt::try_from(i).unwrap();
            plic.interrupt_disable(interrupt);
            PLIC0::set_priority(interrupt, Priority::P0);
        }
        let max_priority = Priority::try_from(Priority::MAX_PRIORITY_NUMBER).unwrap();
        PLIC0::set_threshold(max_priority);
    }

    /// MANDATORY FOR INTERNAL USE OF MACROS
    #[inline(always)]
    pub fn enable() {
        mstatus::set_mie();
        mie::set_mext();
        PLIC0::set_threshold(Priority::P0);
    }

    /// used in bindings macros, we can customize it
    #[inline(always)]
    pub fn enable_source(plic: &mut PLIC0, interrupt: Interrupt, priority: u16) {
        let priority = Priority::try_from(priority).unwrap();
        PLIC0::set_priority(interrupt, priority);
        plic.interrupt_enable(interrupt);
    }
}

/// MANDATORY FOR INTERNAL USE OF MACROS
#[inline(always)]
pub fn run<F>(priority: u8, f: F)
where
    F: FnOnce(),
{
    let priority = Priority::try_from(priority as u16).unwrap();
    let current: Priority = PLIC0::get_threshold();
    PLIC0::set_threshold(priority);
    f();
    PLIC0::set_threshold(current);
}

/// used in bindings macros, we can customize it
#[inline(always)]
pub unsafe fn lock<T, R>(ptr: *mut T, ceiling: u16, f: impl FnOnce(&mut T) -> R) -> R {
    let ceiling = Priority::try_from(ceiling).unwrap();
    let current: Priority = PLIC0::get_threshold();
    PLIC0::set_threshold(ceiling);
    let r = f(&mut *ptr);
    PLIC0::set_threshold(current);
    r
}

/// Sets the given `interrupt` as pending
///
/// MANDATORY FOR INTERNAL USE OF MACROS
pub fn pend<I: InterruptNumber>(interrupt: I) {
    // TODO PLIC does not allow to trigger interrupts manually
    // We will need to fake them using SoftwareInterrupts!
}
