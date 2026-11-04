use core::future::Future;
use core::pin::pin;
use core::task::{Context, Waker};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::Wake;

struct Count(AtomicUsize);

impl Wake for Count {
    fn wake(self: Arc<Self>) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

/// Poll `fut` with two different wakers, run `wake`, and assert that the second waker was woken.
pub(crate) fn assert_wakes_latest_waker(fut: impl Future, wake: impl FnOnce()) {
    let count = Arc::new(Count(AtomicUsize::new(0)));
    let waker = Waker::from(count.clone());
    let mut fut = pin!(fut);

    let mut cx = Context::from_waker(Waker::noop());
    assert!(fut.as_mut().poll(&mut cx).is_pending());
    let mut cx = Context::from_waker(&waker);
    assert!(fut.as_mut().poll(&mut cx).is_pending());

    wake();

    assert_eq!(count.0.load(Ordering::SeqCst), 1);
}
