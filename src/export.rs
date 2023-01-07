pub use bare_metal::CriticalSection;
use core::sync::atomic::{AtomicBool, Ordering};
pub use cortex_m::{
    asm::nop,
    asm::wfi,
    interrupt,
    peripheral::{scb::SystemHandler, DWT, NVIC, SCB, SYST},
    Peripherals,
};

pub mod executor {
    use core::{
        future::Future,
        mem,
        pin::Pin,
        task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
    };

    static WAKER_VTABLE: RawWakerVTable =
        RawWakerVTable::new(waker_clone, waker_wake, waker_wake, waker_drop);

    unsafe fn waker_clone(p: *const ()) -> RawWaker {
        RawWaker::new(p, &WAKER_VTABLE)
    }

    unsafe fn waker_wake(p: *const ()) {
        // The only thing we need from a waker is the function to call to pend the async
        // dispatcher.
        let f: fn() = mem::transmute(p);
        f();
    }

    unsafe fn waker_drop(_: *const ()) {
        // nop
    }

    //============
    // AsyncTaskExecutor

    pub struct AsyncTaskExecutor<F: Future + 'static> {
        task: Option<F>,
    }

    impl<F: Future + 'static> AsyncTaskExecutor<F> {
        pub const fn new() -> Self {
            Self { task: None }
        }

        pub fn is_running(&self) -> bool {
            self.task.is_some()
        }

        pub fn spawn(&mut self, future: F) {
            self.task = Some(future);
        }

        pub fn poll(&mut self, wake: fn()) -> bool {
            if let Some(future) = &mut self.task {
                unsafe {
                    let waker = Waker::from_raw(RawWaker::new(wake as *const (), &WAKER_VTABLE));
                    let mut cx = Context::from_waker(&waker);
                    let future = Pin::new_unchecked(future);

                    match future.poll(&mut cx) {
                        Poll::Ready(_) => {
                            self.task = None;
                            true // Only true if we finished now
                        }
                        Poll::Pending => false,
                    }
                }
            } else {
                false
            }
        }
    }
}

/// Mask is used to store interrupt masks on systems without a BASEPRI register (M0, M0+, M23).
/// It needs to be large enough to cover all the relevant interrupts in use.
/// For M0/M0+ there are only 32 interrupts so we only need one u32 value.
/// For M23 there can be as many as 480 interrupts.
/// Rather than providing space for all possible interrupts, we just detect the highest interrupt in
/// use at compile time and allocate enough u32 chunks to cover them.
#[derive(Copy, Clone)]
pub struct Mask<const M: usize>([u32; M]);

impl<const M: usize> core::ops::BitOrAssign for Mask<M> {
    fn bitor_assign(&mut self, rhs: Self) {
        for i in 0..M {
            self.0[i] |= rhs.0[i];
        }
    }
}

#[cfg(not(have_basepri))]
impl<const M: usize> Mask<M> {
    /// Set a bit inside a Mask.
    const fn set_bit(mut self, bit: u32) -> Self {
        let block = bit / 32;

        if block as usize >= M {
            panic!("Generating masks for thumbv6/thumbv8m.base failed! Are you compiling for thumbv6 on an thumbv7 MCU or using an unsupported thumbv8m.base MCU?");
        }

        let offset = bit - (block * 32);
        self.0[block as usize] |= 1 << offset;
        self
    }
}

#[cfg(have_basepri)]
use cortex_m::register::basepri;

#[cfg(have_basepri)]
#[inline(always)]
pub fn run<F>(priority: u8, f: F)
where
    F: FnOnce(),
{
    if priority == 1 {
        // If the priority of this interrupt is `1` then BASEPRI can only be `0`
        f();
        unsafe { basepri::write(0) }
    } else {
        let initial = basepri::read();
        f();
        unsafe { basepri::write(initial) }
    }
}

#[cfg(not(have_basepri))]
#[inline(always)]
pub fn run<F>(_priority: u8, f: F)
where
    F: FnOnce(),
{
    f();
}

pub struct Barrier {
    inner: AtomicBool,
}

impl Barrier {
    pub const fn new() -> Self {
        Barrier {
            inner: AtomicBool::new(false),
        }
    }

    pub fn release(&self) {
        self.inner.store(true, Ordering::Release);
    }

    pub fn wait(&self) {
        while !self.inner.load(Ordering::Acquire) {
            core::hint::spin_loop()
        }
    }
}

/// Const helper to check architecture
pub const fn have_basepri() -> bool {
    #[cfg(have_basepri)]
    {
        true
    }

    #[cfg(not(have_basepri))]
    {
        false
    }
}

#[inline(always)]
pub fn assert_send<T>()
where
    T: Send,
{
}

#[inline(always)]
pub fn assert_sync<T>()
where
    T: Sync,
{
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
#[cfg(have_basepri)]
#[inline(always)]
pub unsafe fn lock<T, R, const M: usize>(
    ptr: *mut T,
    ceiling: u8,
    nvic_prio_bits: u8,
    _mask: &[Mask<M>; 3],
    f: impl FnOnce(&mut T) -> R,
) -> R {
    if ceiling == (1 << nvic_prio_bits) {
        let r = interrupt::free(|_| f(&mut *ptr));
        r
    } else {
        let current = basepri::read();
        basepri::write(logical2hw(ceiling, nvic_prio_bits));
        let r = f(&mut *ptr);
        basepri::write(current);
        r
    }
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
///   else
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
#[cfg(not(have_basepri))]
#[inline(always)]
pub unsafe fn lock<T, R, const M: usize>(
    ptr: *mut T,
    ceiling: u8,
    _nvic_prio_bits: u8,
    masks: &[Mask<M>; 3],
    f: impl FnOnce(&mut T) -> R,
) -> R {
    if ceiling >= 4 {
        // safe to manipulate outside critical section
        // execute closure under protection of raised system ceiling
        let r = interrupt::free(|_| f(&mut *ptr));
        // safe to manipulate outside critical section
        r
    } else {
        // safe to manipulate outside critical section
        let mask = compute_mask(0, ceiling, masks);
        clear_enable_mask(mask);

        // execute closure under protection of raised system ceiling
        let r = f(&mut *ptr);

        set_enable_mask(mask);

        // safe to manipulate outside critical section
        r
    }
}

#[cfg(not(have_basepri))]
#[inline(always)]
fn compute_mask<const M: usize>(from_prio: u8, to_prio: u8, masks: &[Mask<M>; 3]) -> Mask<M> {
    let mut res = Mask([0; M]);
    masks[from_prio as usize..to_prio as usize]
        .iter()
        .for_each(|m| res |= *m);
    res
}

// enables interrupts
#[cfg(not(have_basepri))]
#[inline(always)]
unsafe fn set_enable_mask<const M: usize>(mask: Mask<M>) {
    for i in 0..M {
        // This check should involve compile time constants and be optimized out.
        if mask.0[i] != 0 {
            (*NVIC::PTR).iser[i].write(mask.0[i]);
        }
    }
}

// disables interrupts
#[cfg(not(have_basepri))]
#[inline(always)]
unsafe fn clear_enable_mask<const M: usize>(mask: Mask<M>) {
    for i in 0..M {
        // This check should involve compile time constants and be optimized out.
        if mask.0[i] != 0 {
            (*NVIC::PTR).icer[i].write(mask.0[i]);
        }
    }
}

#[inline]
#[must_use]
pub fn logical2hw(logical: u8, nvic_prio_bits: u8) -> u8 {
    ((1 << nvic_prio_bits) - logical) << (8 - nvic_prio_bits)
}

#[cfg(have_basepri)]
pub const fn create_mask<const N: usize, const M: usize>(_: [u32; N]) -> Mask<M> {
    Mask([0; M])
}

#[cfg(not(have_basepri))]
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

#[cfg(have_basepri)]
pub const fn compute_mask_chunks<const L: usize>(_: [u32; L]) -> usize {
    0
}

/// Compute the number of u32 chunks needed to store the Mask value.
/// On M0, M0+ this should always end up being 1.
/// On M23 we will pick a number that allows us to store the highest index used by the code.
/// This means the amount of overhead will vary based on the actually interrupts used by the code.
#[cfg(not(have_basepri))]
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

#[cfg(have_basepri)]
pub const fn no_basepri_panic() {
    // For non-v6 all is fine
}

#[cfg(not(have_basepri))]
pub const fn no_basepri_panic() {
    panic!("Exceptions with shared resources are not allowed when compiling for thumbv6 or thumbv8m.base. Use local resources or `#[lock_free]` shared resources");
}
