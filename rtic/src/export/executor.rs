use super::atomic::{AtomicBool, Ordering};
use core::{
    cell::UnsafeCell,
    convert::Infallible,
    future::Future,
    mem::{self, MaybeUninit},
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
    let f: fn() = unsafe { mem::transmute(p) };
    f();
}

unsafe fn waker_drop(_: *const ()) {
    // nop
}

//============
// Naming a task's future

/// Binds the future type of an `async fn` as an associated type.
///
/// An `async fn`'s future type cannot be written down, so a `static` holding one cannot be
/// declared directly. Naming it as `<F as ExecFn<Args>>::Fut` for the function's own type is
/// enough to ask for its size and alignment in a `const`, which is what [`ExecutorHolder`]
/// needs.
pub trait ExecFn<Args>: Copy {
    /// The future the function returns.
    type Fut: Future + 'static;
}

macro_rules! exec_fn_impl {
    ($($Tn:ident),*) => {
        impl<F, Fut, $($Tn,)*> ExecFn<($($Tn,)*)> for F
        where
            F: Copy + FnOnce($($Tn,)*) -> Fut,
            Fut: Future + 'static,
        {
            type Fut = Fut;
        }
    };
}

// A task takes its context plus its own inputs, so this bounds inputs at one fewer.
exec_fn_impl!();
exec_fn_impl!(T0);
exec_fn_impl!(T0, T1);
exec_fn_impl!(T0, T1, T2);
exec_fn_impl!(T0, T1, T2, T3);
exec_fn_impl!(T0, T1, T2, T3, T4);
exec_fn_impl!(T0, T1, T2, T3, T4, T5);
exec_fn_impl!(T0, T1, T2, T3, T4, T5, T6);
exec_fn_impl!(T0, T1, T2, T3, T4, T5, T6, T7);
exec_fn_impl!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
exec_fn_impl!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
exec_fn_impl!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
exec_fn_impl!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
exec_fn_impl!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
exec_fn_impl!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
exec_fn_impl!(
    T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14
);
exec_fn_impl!(
    T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15
);

//============
// Storage for an executor whose future cannot be named

/// The size a task's executor needs, for [`ExecutorHolder`]'s first parameter.
pub const fn exec_size<F, Args, Fut>(_f: F) -> usize
where
    F: ExecFn<Args, Fut = Fut>,
    Fut: Future + 'static,
{
    size_of::<AsyncTaskExecutor<Fut>>()
}

/// The alignment a task's executor needs, for [`ExecutorHolder`]'s second parameter.
pub const fn exec_align<F, Args, Fut>(_f: F) -> usize
where
    F: ExecFn<Args, Fut = Fut>,
    Fut: Future + 'static,
{
    align_of::<AsyncTaskExecutor<Fut>>()
}

/// An executor for a task, to be transmuted into the [`ExecutorHolder`] that stores it.
pub const fn exec_new<F, Args, Fut>(_f: F) -> AsyncTaskExecutor<Fut>
where
    F: ExecFn<Args, Fut = Fut>,
    Fut: Future + 'static,
{
    AsyncTaskExecutor::new()
}

/// Storage for one [`AsyncTaskExecutor`], sized and aligned for a future that cannot be named.
///
/// Declared as bytes so the `static` needs no type parameter, and initialized by transmuting an
/// [`exec_new`]. Both flags are false in that image, so this lands in `.bss` and costs no
/// initializer.
#[allow(private_bounds)]
#[repr(C)]
pub struct ExecutorHolder<const SIZE: usize, const ALIGN: usize>
where
    Align<ALIGN>: Alignment,
{
    data: UnsafeCell<[MaybeUninit<u8>; SIZE]>,
    align: Align<ALIGN>,
}

unsafe impl<const SIZE: usize, const ALIGN: usize> Send for ExecutorHolder<SIZE, ALIGN> where
    Align<ALIGN>: Alignment
{
}

unsafe impl<const SIZE: usize, const ALIGN: usize> Sync for ExecutorHolder<SIZE, ALIGN> where
    Align<ALIGN>: Alignment
{
}

/// Reads back the executor an [`ExecutorHolder`] was initialized with.
///
/// # Safety
///
/// `holder` must have been initialized by transmuting [`exec_new`] applied to the same `_f`.
#[allow(private_bounds)]
#[inline(always)]
pub unsafe fn exec_from_holder<F, Args, Fut, const SIZE: usize, const ALIGN: usize>(
    _f: F,
    holder: &'static ExecutorHolder<SIZE, ALIGN>,
) -> &'static AsyncTaskExecutor<Fut>
where
    F: ExecFn<Args, Fut = Fut>,
    Fut: Future + 'static,
    Align<ALIGN>: Alignment,
{
    // Guard against a change to the `executor_decl` macro making it silently transmute the wrong bytes.
    const {
        assert!(SIZE == size_of::<AsyncTaskExecutor<Fut>>());
        assert!(ALIGN == align_of::<AsyncTaskExecutor<Fut>>());
    }

    unsafe { &*holder.data.get().cast() }
}

#[allow(private_bounds)]
#[repr(transparent)]
pub struct Align<const N: usize>([<Self as Alignment>::Archetype; 0])
where
    Self: Alignment;

trait Alignment {
    /// A zero-sized type of particular alignment.
    type Archetype: Copy + Eq + PartialEq + Send + Sync + Unpin;
}

macro_rules! aligns {
    ($($AlignX:ident: $n:literal,)*) => {
        $(
            #[derive(Copy, Clone, Eq, PartialEq)]
            #[repr(align($n))]
            struct $AlignX {}

            impl Alignment for Align<$n> {
                type Archetype = $AlignX;
            }
        )*
    };
}

aligns!(
    Align1: 1,
    Align2: 2,
    Align4: 4,
    Align8: 8,
    Align16: 16,
    Align32: 32,
    Align64: 64,
    Align128: 128,
    Align256: 256,
    Align512: 512,
    Align1024: 1024,
    Align2048: 2048,
    Align4096: 4096,
    Align8192: 8192,
    Align16384: 16384,
);

//============
// AsyncTaskExecutor

/// Executor for an async task.
pub struct AsyncTaskExecutor<F: Future + 'static> {
    // `task` is protected by the `running` flag.
    task: UnsafeCell<MaybeUninit<F>>,
    running: AtomicBool,
    pending: AtomicBool,
}

unsafe impl<F: Future + 'static> Sync for AsyncTaskExecutor<F> {}

impl<F: Future + 'static> Default for AsyncTaskExecutor<F> {
    fn default() -> Self {
        Self::new()
    }
}

impl<F: Future + 'static> AsyncTaskExecutor<F> {
    /// Create a new executor.
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            task: UnsafeCell::new(MaybeUninit::uninit()),
            running: AtomicBool::new(false),
            pending: AtomicBool::new(false),
        }
    }

    /// Check if there is an active task in the executor.
    #[inline(always)]
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Checks if a waker has pended the executor and simultaneously clears the flag.
    #[inline(always)]
    fn check_and_clear_pending(&self) -> bool {
        // Ordering::Acquire to enforce that update of task is visible to poll
        self.pending
            .compare_exchange(true, false, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    // Used by wakers to indicate that the executor needs to run.
    #[inline(always)]
    pub fn set_pending(&self) {
        self.pending.store(true, Ordering::Release);
    }

    /// Allocate the executor. To use with `spawn`.
    #[inline(always)]
    pub unsafe fn try_allocate(&self) -> bool {
        // Try to reserve the executor for a future.
        self.running
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
    }

    /// Spawn a future
    #[inline(always)]
    pub unsafe fn spawn(&self, future: F) {
        // This unsafe is protected by `running` being false and the atomic setting it to true.
        unsafe {
            self.task.get().write(MaybeUninit::new(future));
        }
        self.set_pending();
    }

    #[inline(always)]
    pub const fn waker(&self, wake: fn()) -> Waker {
        unsafe { Waker::from_raw(RawWaker::new(wake as *const (), &WAKER_VTABLE)) }
    }

    /// Poll the future in the executor.
    #[inline(always)]
    pub fn poll(&self, wake: fn()) {
        if self.is_running() && self.check_and_clear_pending() {
            let waker = self.waker(wake);
            let mut cx = Context::from_waker(&waker);
            let future = unsafe { Pin::new_unchecked(&mut *(self.task.get() as *mut F)) };

            match future.poll(&mut cx) {
                Poll::Ready(_) => {
                    self.running.store(false, Ordering::Release);
                }
                Poll::Pending => {}
            }
        }
    }
}

/// This function is used to assert that tasks that
/// return `!` are backed by functions that return `!`.
///
/// The only type that implements `Into<Infallible>` is
/// `!`.
pub fn assert_task_diverges<F, I>(f: F) -> F
where
    I: Into<Infallible>,
    F: Future<Output = I>,
{
    f
}
