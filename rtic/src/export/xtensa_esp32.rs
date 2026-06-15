use core::arch::asm;

pub struct Peripherals;

impl Peripherals {
    pub unsafe fn steal() -> Self {
        Self
    }
}

//not using bare metal INTENABLE so we don't interfere with esp-hal's
//INTENABLE setup done by init_vectoring() during __pre_init
pub mod interrupt {
    use super::asm;

    //masks all user interrupts
    pub fn disable() {
        unsafe { asm!("rsil {0}, 5", out(reg) _, options(nostack)) }
    }

    //called right after init, sets INTLEVEL to 0
    pub unsafe fn enable() {
        unsafe { asm!("rsil {0}, 0", out(reg) _, options(nostack)) }
    }
}

//atomically raise PS.INTLEVEL to `level` and return the full old PS register
//rsil is the only atomic read-modify-write for PS.INTLEVEL (see esp-hal), but
//it requires a compile-time constant level
//returning the whole PS (not just INTLEVEL bits) lets `restore_ps` do a full restore
#[inline(always)]
fn rsil(level: u8) -> u32 {
    let old_ps: u32;
    unsafe {
        match level {
            1 => asm!("rsil {0}, 1", out(reg) old_ps, options(nostack)),
            2 => asm!("rsil {0}, 2", out(reg) old_ps, options(nostack)),
            3 => asm!("rsil {0}, 3", out(reg) old_ps, options(nostack)),
            4 => asm!("rsil {0}, 4", out(reg) old_ps, options(nostack)),
            5 => asm!("rsil {0}, 5", out(reg) old_ps, options(nostack)),
            6 => asm!("rsil {0}, 6", out(reg) old_ps, options(nostack)),
            7 => asm!("rsil {0}, 7", out(reg) old_ps, options(nostack)),
            _ => asm!("rsil {0}, 0", out(reg) old_ps, options(nostack)),
        }
    }
    old_ps
}

#[inline(always)]
unsafe fn restore_ps(ps: u32) {
    unsafe {
        asm!("wsr {0}, PS", "rsync", in(reg) ps, options(nostack));
    }
}

//execute f at some priority
//
//raises PS.INTLEVEL to said priority so that any interrupt at that level or below
//cannot preempt the running task
//restores the previous level on return
//unfortunately, we can only call priorities 1-3 with rust...
//need asm for 4+
//smth smth c api?
#[inline(always)]
pub fn run<F: FnOnce()>(priority: u8, f: F) {
    let old_ps = rsil(priority);
    f();
    unsafe { restore_ps(old_ps) };
}

//note that we can lock even above priority level 3
#[inline(always)]
pub unsafe fn lock<T, R>(ptr: *mut T, ceiling: u8, f: impl FnOnce(&mut T) -> R) -> R {
    let old_ps = rsil(ceiling);
    let r = f(unsafe { &mut *ptr });
    unsafe { restore_ps(old_ps) };
    r
}

//interrupts 7 and 29 are software interrupts
#[inline(always)]
pub fn pend(int: esp32::Interrupt) {
    let mask: u32 = match int {
        esp32::Interrupt::FROM_CPU_INTR0 => 1 << 7,
        esp32::Interrupt::FROM_CPU_INTR1 => 1 << 29,
        _ => return,
    };
    unsafe { asm!("wsr.intset {0}", in(reg) mask, options(nostack)) };
}

#[inline(always)]
pub fn unpend(_int: esp32::Interrupt) {}
