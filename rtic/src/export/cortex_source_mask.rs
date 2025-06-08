pub use cortex_m::{
    Peripherals,
    asm::wfi,
    interrupt,
    peripheral::{DWT, NVIC, SCB, SYST, scb::SystemHandler},
};

#[cfg(not(any(feature = "thumbv6-backend", feature = "thumbv8base-backend")))]
compile_error!(
    "Building for Cortex-M with source masking, but 'thumbv6-backend' or 'thumbv8base-backend' backend not selected"
);

/// Mask is used to store interrupt masks on systems without a BASEPRI register (M0, M0+, M23).
/// It needs to be large enough to cover all the relevant interrupts in use.
/// For M0/M0+ there are only 32 interrupts so we only need one u32 value.
/// For M23 there can be as many as 480 interrupts.
/// Rather than providing space for all possible interrupts, we just detect the highest interrupt in
/// use at compile time and allocate enough u32 chunks to cover them.
#[derive(Copy, Clone)]
pub struct Mask<const M: usize>([u32; M]);

pub const fn create_mask<const N: usize, const M: usize>(list_of_shifts: [u32; N]) -> Mask<M> {
    let mut mask = Mask([0; M]);
    let mut i = 0;

    while i < N {
        let shift = list_of_shifts[i];
        i += 1;
        mask = mask.set_bit(shift);
    }

    mask
}

/// Compute the number of u32 chunks needed to store the Mask value.
/// On M0, M0+ this should always end up being 1.
/// On M23 we will pick a number that allows us to store the highest index used by the code.
/// This means the amount of overhead will vary based on the actually interrupts used by the code.
pub const fn compute_mask_chunks<const L: usize>(ids: [u32; L]) -> usize {
    let mut max: usize = 0;
    let mut i = 0;

    while i < L {
        let id = ids[i] as usize;
        i += 1;

        if id > max {
            max = id;
        }
    }
    (max + 32) / 32
}

impl<const M: usize> Mask<M> {
    /// Set a bit inside a Mask.
    const fn set_bit(mut self, bit: u32) -> Self {
        let block = bit / 32;

        if block as usize >= M {
            panic!(
                "Generating masks for thumbv6/thumbv8m.base failed! Are you compiling for thumbv6 on an thumbv7 MCU or using an unsupported thumbv8m.base MCU?"
            );
        }

        let offset = bit - (block * 32);
        self.0[block as usize] |= 1 << offset;
        self
    }
}

#[inline(always)]
pub fn run<F>(_priority: u8, f: F)
where
    F: FnOnce(),
{
    f();
}

/// Lock implementation using interrupt masking
///
/// # Safety
///
/// The system ceiling is raised from current to ceiling
/// by computing a 32 bit `mask` (1 bit per interrupt)
/// 1: ceiling >= priority > current
/// 0: else
///
/// On CS entry, `clear_enable_mask(mask)` disables interrupts
/// On CS exit,  `set_enable_mask(mask)` re-enables interrupts
///
/// The priority.set/priority.get can safely be outside the CS
/// as being a context local cell (not affected by preemptions).
/// It is merely used in order to omit masking in case
/// current priority >= ceiling.
///
/// Dereferencing a raw pointer is done safely inside the CS
///
/// Lock Efficiency:
/// Early experiments validate (sub)-zero cost for CS implementation
/// (Sub)-zero as:
/// - Either zero OH (lock optimized out), or
/// - Amounting to an optimal assembly implementation
///   - if ceiling == (1 << nvic_prio_bits)
///     - we execute the closure in a global critical section (interrupt free)
///     - CS entry cost, single write to core register
///     - CS exit cost, single write to core register
///   - else
///     - The `mask` value is folded to a constant at compile time
///     - CS entry, single write of the 32 bit `mask` to the `icer` register
///     - CS exit, single write of the 32 bit `mask` to the `iser` register
/// - priority.set/get optimized out (their effect not)
/// - On par or better than any hand written implementation of SRP
///
/// Limitations:
/// Current implementation does not allow for tasks with shared resources
/// to be bound to exception handlers, as these cannot be masked in HW.
///
/// Possible solutions:
/// - Mask exceptions by global critical sections (interrupt::free)
/// - Temporary lower exception priority
///
/// These possible solutions are set goals for future work
#[inline(always)]
pub unsafe fn lock<T, R, const M: usize>(
    ptr: *mut T,
    ceiling: u8,
    masks: &[Mask<M>; 3],
    f: impl FnOnce(&mut T) -> R,
) -> R {
    unsafe {
        if ceiling >= 4 {
            // safe to manipulate outside critical section
            // execute closure under protection of raised system ceiling

            // safe to manipulate outside critical section
            critical_section::with(|_| f(&mut *ptr))
        } else {
            // safe to manipulate outside critical section
            let mask = compute_mask(0, ceiling, masks);
            let old_mask = read_mask(mask);
            clear_enable_mask(mask);

            // execute closure under protection of raised system ceiling
            let r = f(&mut *ptr);

            set_enable_mask(mask, old_mask);

            // safe to manipulate outside critical section
            r
        }
    }
}

#[inline(always)]
pub const fn compute_mask<const M: usize>(
    from_prio: u8,
    to_prio: u8,
    masks: &[Mask<M>; 3],
) -> Mask<M> {
    let mut res = Mask([0; M]);

    let mut idx = from_prio as usize;

    while idx < to_prio as usize {
        let mut i = 0;

        while i < M {
            //self.0[i] |= rhs.0[i];
            res.0[i] |= masks[idx].0[i];
            i += 1;
        }

        idx += 1;
    }

    // old code (non const)
    // masks[from_prio as usize..to_prio as usize]
    //     .iter()
    //     .for_each(|m| res |= *m);

    res
}

// enables interrupts
#[inline(always)]
unsafe fn read_mask<const M: usize>(mask: Mask<M>) -> Mask<M> {
    let mut out = Mask([0; M]);

    for i in 0..M {
        // This check should involve compile time constants and be optimized out.
        if mask.0[i] != 0 {
            out.0[i] = unsafe { (*NVIC::PTR).iser[i].read() };
        }
    }

    out
}

// enables interrupts
#[inline(always)]
unsafe fn set_enable_mask<const M: usize>(mask: Mask<M>, old_mask: Mask<M>) {
    for i in 0..M {
        // This check should involve compile time constants and be optimized out.
        if mask.0[i] != 0 {
            unsafe {
                (*NVIC::PTR).iser[i].write(old_mask.0[i]);
            }
        }
    }
}

// disables interrupts
#[inline(always)]
unsafe fn clear_enable_mask<const M: usize>(mask: Mask<M>) {
    for i in 0..M {
        // This check should involve compile time constants and be optimized out.
        if mask.0[i] != 0 {
            unsafe {
                (*NVIC::PTR).icer[i].write(mask.0[i]);
            }
        }
    }
}
