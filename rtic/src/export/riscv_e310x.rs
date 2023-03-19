use riscv::riscv::register::{mie, mstatus};

pub use e310x::{
    plic::{Interrupt, Priority}, // target-specific PLIC-related enums
    Peripherals,                 // Contains a PLIC field
};
pub use riscv::{
    asm::nop,
    peripheral::plic::{InterruptNumber, PriorityLevel}, // Generic PLIC-related traits
};

pub mod interrupt {

    #[inline(always)]
    pub fn disable() {
        mstatus::clear_mie();
        mie::clear_mext();
        reset_plic();
    }

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

    #[inline(always)]
    pub fn enable() {
        mstatus::set_mie();
        mie::set_mext();
        PLIC0::set_threshold(Priority::P0);
    }

    #[inline(always)]
    pub fn enable_source(plic: &mut PLIC0, interrupt: Interrupt, priority: u16) {
        let priority = Priority::try_from(priority).unwrap();
        PLIC0::set_priority(interrupt, priority);
        plic.interrupt_enable(interrupt);
    }
}

#[inline(always)]
pub fn run<F>(priority: u16, f: F)
where
    F: FnOnce(),
{
    let priority = Priority::try_from(priority).unwrap();
    let current: Priority = PLIC0::get_threshold();
    PLIC0::set_threshold(priority);
    f();
    PLIC0::set_threshold(current);
}

#[inline(always)]
pub unsafe fn lock<T, R>(ptr: *mut T, ceiling: u16, f: impl FnOnce(&mut T) -> R) -> R {
    let priority = Priority::try_from(ceiling).unwrap();
    let current: Priority = PLIC0::get_threshold();
    PLIC0::set_threshold(priority);
    let r = f(&mut *ptr);
    PLIC0::set_threshold(current);
    r
}
