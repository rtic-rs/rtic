use super::atomic::{AtomicBool, Ordering};
use core::{
    cell::UnsafeCell,
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
    // `task` is protected by the `running` flag.
    task: UnsafeCell<MaybeUninit<F>>,
    running: AtomicBool,
    pending: AtomicBool,
}

unsafe impl<F: Future> Sync for AsyncTaskExecutor<F> {}

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

    /// Poll the future in the executor.
    #[inline(always)]
    pub fn poll(&self, wake: fn()) {
        if self.is_running() && self.check_and_clear_pending() {
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
