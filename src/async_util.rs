//! Async support for RTIC

use core::{
    future::Future,
    mem,
    pin::Pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

//=============
// Waker

///
pub static WAKER_VTABLE: RawWakerVTable =
    RawWakerVTable::new(waker_clone, waker_wake, waker_wake, waker_drop);

unsafe fn waker_clone(p: *const ()) -> RawWaker {
    RawWaker::new(p, &WAKER_VTABLE)
}

unsafe fn waker_wake(p: *const ()) {
    let f: fn() = mem::transmute(p);
    f();
}

unsafe fn waker_drop(_: *const ()) {
    // nop
}

//============
// Task

///
pub enum Task<F: Future + 'static> {
    ///
    Idle,

    ///
    Running(F),

    ///
    Done(F::Output),
}

impl<F: Future + 'static> Task<F> {
    ///
    pub const fn new() -> Self {
        Self::Idle
    }

    ///
    pub fn spawn(&mut self, future: impl FnOnce() -> F) {
        *self = Task::Running(future());
    }

    ///
    pub unsafe fn poll(&mut self, wake: fn()) {
        match self {
            Task::Idle => {}
            Task::Running(future) => {
                let future = Pin::new_unchecked(future);
                let waker_data: *const () = mem::transmute(wake);
                let waker = Waker::from_raw(RawWaker::new(waker_data, &WAKER_VTABLE));
                let mut cx = Context::from_waker(&waker);

                match future.poll(&mut cx) {
                    Poll::Ready(r) => *self = Task::Done(r),
                    Poll::Pending => {}
                };
            }
            Task::Done(_) => {}
        }
    }
}
