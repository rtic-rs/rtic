use super::atomic::{AtomicBool, AtomicPtr, Ordering};
use core::{
    cell::UnsafeCell,
    future::Future,
    mem::{self, ManuallyDrop, MaybeUninit},
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

/// Pointer to executor holder.
pub struct AsyncTaskExecutorPtr {
    // Void pointer.
    ptr: AtomicPtr<()>,
}

impl AsyncTaskExecutorPtr {
    pub const fn new() -> Self {
        Self {
            ptr: AtomicPtr::new(core::ptr::null_mut()),
        }
    }

    #[inline(always)]
    pub fn set_in_main<F: Future>(&self, executor: &ManuallyDrop<AsyncTaskExecutor<F>>) {
        self.ptr.store(executor as *const _ as _, Ordering::Relaxed);
    }

    #[inline(always)]
    pub fn get(&self) -> *const () {
        self.ptr.load(Ordering::Relaxed)
    }
}

impl Default for AsyncTaskExecutorPtr {
    fn default() -> Self {
        Self::new()
    }
}

/// Executor for an async task.
pub struct AsyncTaskExecutor<F: Future> {
    // `task` is protected by the `running` flag.
    task: UnsafeCell<MaybeUninit<F>>,
    running: AtomicBool,
    pending: AtomicBool,
}

unsafe impl<F: Future> Sync for AsyncTaskExecutor<F> {}

macro_rules! new_n_args {
    ($name:ident, $($t:ident),*) => {
        #[inline(always)]
        pub fn $name<$($t,)* Fun: Fn($($t,)*) -> F>(_f: Fun) -> Self {
            Self::new()
        }
    };
}

macro_rules! from_ptr_n_args {
    ($name:ident, $($t:ident),*) => {
        #[inline(always)]
        pub unsafe fn $name<$($t,)* Fun: Fn($($t,)*) -> F>(_f: Fun, ptr: &AsyncTaskExecutorPtr) -> &Self {
            &*(ptr.get() as *const _)
        }
    };
}

impl<F: Future> Default for AsyncTaskExecutor<F> {
    fn default() -> Self {
        Self::new()
    }
}

impl<F: Future> AsyncTaskExecutor<F> {
    /// Create a new executor.
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            task: UnsafeCell::new(MaybeUninit::uninit()),
            running: AtomicBool::new(false),
            pending: AtomicBool::new(false),
        }
    }

    // Support for up to 16 arguments on async functions. Should be
    // enough for now, else extend this list.
    new_n_args!(new_0_args,);
    new_n_args!(new_1_args, A1);
    new_n_args!(new_2_args, A1, A2);
    new_n_args!(new_3_args, A1, A2, A3);
    new_n_args!(new_4_args, A1, A2, A3, A4);
    new_n_args!(new_5_args, A1, A2, A3, A4, A5);
    new_n_args!(new_6_args, A1, A2, A3, A4, A5, A6);
    new_n_args!(new_7_args, A1, A2, A3, A4, A5, A6, A7);
    new_n_args!(new_8_args, A1, A2, A3, A4, A5, A6, A7, A8);
    new_n_args!(new_9_args, A1, A2, A3, A4, A5, A6, A7, A8, A9);
    new_n_args!(new_10_args, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10);
    new_n_args!(new_11_args, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11);
    #[rustfmt::skip]
    new_n_args!(new_12_args, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12);
    #[rustfmt::skip]
    new_n_args!(new_13_args, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13);
    #[rustfmt::skip]
    new_n_args!(new_14_args, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14);
    #[rustfmt::skip]
    new_n_args!(new_15_args, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15);
    #[rustfmt::skip]
    new_n_args!(new_16_args, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16);

    from_ptr_n_args!(from_ptr_0_args,);
    from_ptr_n_args!(from_ptr_1_args, A1);
    from_ptr_n_args!(from_ptr_2_args, A1, A2);
    from_ptr_n_args!(from_ptr_3_args, A1, A2, A3);
    from_ptr_n_args!(from_ptr_4_args, A1, A2, A3, A4);
    from_ptr_n_args!(from_ptr_5_args, A1, A2, A3, A4, A5);
    from_ptr_n_args!(from_ptr_6_args, A1, A2, A3, A4, A5, A6);
    from_ptr_n_args!(from_ptr_7_args, A1, A2, A3, A4, A5, A6, A7);
    from_ptr_n_args!(from_ptr_8_args, A1, A2, A3, A4, A5, A6, A7, A8);
    from_ptr_n_args!(from_ptr_9_args, A1, A2, A3, A4, A5, A6, A7, A8, A9);
    from_ptr_n_args!(from_ptr_10_args, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10);
    #[rustfmt::skip]
    from_ptr_n_args!(from_ptr_11_args, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11);
    #[rustfmt::skip]
    from_ptr_n_args!(from_ptr_12_args, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12);
    #[rustfmt::skip]
    from_ptr_n_args!(from_ptr_13_args, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13);
    #[rustfmt::skip]
    from_ptr_n_args!(from_ptr_14_args, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14);
    #[rustfmt::skip]
    from_ptr_n_args!(from_ptr_15_args, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15);
    #[rustfmt::skip]
    from_ptr_n_args!(from_ptr_16_args, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16);

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
