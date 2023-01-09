use core::{
    cell::UnsafeCell,
    future::Future,
    mem::{self, MaybeUninit},
    pin::Pin,
    sync::atomic::{AtomicBool, Ordering},
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

/// Executor for an async task.
pub struct AsyncTaskExecutor<F: Future> {
    // `task` is proteced by the `running` flag.
    task: UnsafeCell<MaybeUninit<F>>,
    running: AtomicBool,
    pending: AtomicBool,
}

unsafe impl<F: Future> Sync for AsyncTaskExecutor<F> {}

impl<F: Future> AsyncTaskExecutor<F> {
    /// Create a new executor.
    pub const fn new() -> Self {
        Self {
            task: UnsafeCell::new(MaybeUninit::uninit()),
            running: AtomicBool::new(false),
            pending: AtomicBool::new(false),
        }
    }

    /// Check if there is an active task in the executor.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Checks if a waker has pended the executor.
    pub fn is_pending(&self) -> bool {
        self.pending.load(Ordering::Relaxed)
    }

    // Used by wakers to indicate that the executor needs to run.
    pub fn set_pending(&self) {
        self.pending.store(true, Ordering::Release);
    }

    /// Try to reserve the executor for a future.
    /// Used in conjunction with `spawn_unchecked` to reserve the executor before spawning.
    ///
    /// This could have been joined with `spawn_unchecked` for a complete safe API, however the
    /// codegen needs to see if the reserve fails so it can give back input parameters. If spawning
    /// was done within the same call the input parameters would be lost and could not be returned.
    pub fn try_reserve(&self) -> bool {
        self.running
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
    }

    /// Spawn a future, only valid to do after `try_reserve` succeeds.
    pub unsafe fn spawn_unchecked(&self, future: F) {
        debug_assert!(self.running.load(Ordering::Relaxed));

        self.task.get().write(MaybeUninit::new(future));
    }

    /// Poll the future in the executor.
    pub fn poll(&self, wake: fn()) {
        if self.is_running() {
            let waker = unsafe { Waker::from_raw(RawWaker::new(wake as *const (), &WAKER_VTABLE)) };
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
