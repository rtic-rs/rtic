pub use esp32c3::{Peripherals, Interrupt};
pub use esp32c3_hal::interrupt as hal_interrupt; //high level peripheral interrupt access
pub use esp32c3_hal::riscv::interrupt; //low level interrupt enable/disable
use esp32c3_hal::interrupt::Priority; //need this for setting priority since the method takes an object and not a int
use esp32c3::INTERRUPT_CORE0; //priority threshold control
use rtt_target::rprintln;


#[inline(always)]
pub fn run<F>(priority: u8, f: F)
where
    F: FnOnce(),
{
    if priority == 1 {
        //if priority is 1, priority thresh should be 1
        f();
        unsafe {
            (*INTERRUPT_CORE0::ptr()).
            cpu_int_thresh.
            write(|w|{w.cpu_int_thresh().bits(1)});
        }
    } else {
        //just a read so safe
        let initial = unsafe{
            (*INTERRUPT_CORE0::ptr())
            .cpu_int_thresh.read()
            .cpu_int_thresh()
            .bits()
        };
        f();
        //write back the old value, safe
        unsafe {
            (*INTERRUPT_CORE0::ptr()).
            cpu_int_thresh.
            write(|w|{
                w.cpu_int_thresh().
                bits(initial)
            });
        }
    }
}

/// Lock implementation using BASEPRI and global Critical Section (CS)
///
/// # Safety
///
/// The system ceiling is raised from current to ceiling
/// by either
/// - raising the BASEPRI to the ceiling value, or
/// - disable all interrupts in case we want to
///   mask interrupts with maximum priority
///
/// Dereferencing a raw pointer inside CS
///
/// The priority.set/priority.get can safely be outside the CS
/// as being a context local cell (not affected by preemptions).
/// It is merely used in order to omit masking in case current
/// priority is current priority >= ceiling.
///
/// Lock Efficiency:
/// Experiments validate (sub)-zero cost for CS implementation
/// (Sub)-zero as:
/// - Either zero OH (lock optimized out), or
/// - Amounting to an optimal assembly implementation
///   - The BASEPRI value is folded to a constant at compile time
///   - CS entry, single assembly instruction to write BASEPRI
///   - CS exit, single assembly instruction to write BASEPRI
///   - priority.set/get optimized out (their effect not)
/// - On par or better than any handwritten implementation of SRP
///
/// Limitations:
/// The current implementation reads/writes BASEPRI once
/// even in some edge cases where this may be omitted.
/// Total OH of per task is max 2 clock cycles, negligible in practice
/// but can in theory be fixed.
///
#[inline(always)]
pub unsafe fn lock<T, R>(
    ptr: *mut T,
    ceiling: u8,
    f: impl FnOnce(&mut T) -> R,
) -> R {
    
    if ceiling == (15) { //turn off interrupts completely, were at max prio
        let r = critical_section::with(|_| f(&mut *ptr));
        r
    } else {

        let current = unsafe{
            (*INTERRUPT_CORE0::ptr())
            .cpu_int_thresh
            .read()
            .cpu_int_thresh()
            .bits()
        };

        unsafe{(*INTERRUPT_CORE0::ptr()).cpu_int_thresh.write(|w|w.cpu_int_thresh().bits(ceiling + 1))}     //esp32c3 lets interrupts with prio equal to threshold through so we up it by one
        let r = f(&mut *ptr);
        unsafe{(*INTERRUPT_CORE0::ptr()).cpu_int_thresh.write(|w|w.cpu_int_thresh().bits(current))}
        r
    }
}

pub fn int_to_prio(int:u8) -> Priority{
    match(int){
        0 => Priority::None,
        1 => Priority::Priority1,
        2 => Priority::Priority2,
        3 => Priority::Priority3,
        4 => Priority::Priority4,
        5 => Priority::Priority5,
        6 => Priority::Priority6,
        7 => Priority::Priority7,
        8 => Priority::Priority8,
        9 => Priority::Priority9,
        10 => Priority::Priority10,
        11 => Priority::Priority11,
        12 => Priority::Priority12,
        13 => Priority::Priority13,
        14 => Priority::Priority14,
        15 => Priority::Priority15,
        _ => panic!(), //unsupported priority supplied, so best approach is to panic i think.
    }
}