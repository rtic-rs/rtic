//! A test that verifies the correctness of the [`TimerQueue`].
//!
//! To run this test, you need to activate the `critical-section/std` feature.

use cassette::Cassette;
use parking_lot::Mutex;
use rtic_time::timer_queue::{TimerQueue, TimerQueueBackend};

mod peripheral {
    use parking_lot::Mutex;
    use std::{
        sync::atomic::{AtomicU64, Ordering},
        task::{Poll, Waker},
    };

    use super::TestMonoBackend;

    static NOW: AtomicU64 = AtomicU64::new(0);
    static WAKERS: Mutex<Vec<Waker>> = Mutex::new(Vec::new());

    pub fn tick() -> bool {
        NOW.fetch_add(1, Ordering::Release);

        let had_wakers = !WAKERS.lock().is_empty();
        // Wake up all things waiting for a specific time to happen.
        for waker in WAKERS.lock().drain(..) {
            waker.wake_by_ref();
        }

        let had_interrupt = TestMonoBackend::tick(false);

        had_interrupt || had_wakers
    }

    pub fn now() -> u64 {
        NOW.load(Ordering::Acquire)
    }

    pub async fn wait_until(time: u64) {
        core::future::poll_fn(|ctx| {
            if now() >= time {
                Poll::Ready(())
            } else {
                WAKERS.lock().push(ctx.waker().clone());
                Poll::Pending
            }
        })
        .await;
    }
}

static COMPARE: Mutex<Option<u64>> = Mutex::new(None);
static TIMER_QUEUE: TimerQueue<TestMonoBackend> = TimerQueue::new();

pub struct TestMonoBackend;

impl TestMonoBackend {
    pub fn tick(force_interrupt: bool) -> bool {
        let now = peripheral::now();

        let compare_reached = Some(now) == Self::compare();
        let interrupt = compare_reached || force_interrupt;

        if interrupt {
            unsafe {
                TestMonoBackend::timer_queue().on_monotonic_interrupt();
            }
            true
        } else {
            false
        }
    }

    pub fn compare() -> Option<u64> {
        COMPARE.lock().clone()
    }
}

impl TestMonoBackend {
    fn init() {
        Self::timer_queue().initialize(Self);
    }
}

impl TimerQueueBackend for TestMonoBackend {
    type Ticks = u64;

    fn now() -> Self::Ticks {
        peripheral::now()
    }

    fn set_compare(instant: Self::Ticks) {
        *COMPARE.lock() = Some(instant);
    }

    fn clear_compare_flag() {}

    fn pend_interrupt() {
        Self::tick(true);
    }

    fn timer_queue() -> &'static TimerQueue<Self> {
        &TIMER_QUEUE
    }
}

#[test]
fn timer_queue() {
    TestMonoBackend::init();
    let start = 0;

    let build_delay_test = |pre_delay: Option<u64>, delay: u64| {
        let total = if let Some(pre_delay) = pre_delay {
            pre_delay + delay
        } else {
            delay
        };

        async move {
            // A `pre_delay` simulates a delay in scheduling,
            // without the `pre_delay` being present in the timer
            // queue
            if let Some(pre_delay) = pre_delay {
                peripheral::wait_until(start + pre_delay).await;
            }

            TestMonoBackend::timer_queue().delay(delay).await;

            let elapsed = peripheral::now() - start;
            println!("{total} ticks delay reached after {elapsed} ticks");

            // Expect a delay of one longer, to compensate for timer uncertainty
            if elapsed != total + 1 {
                panic!("{total} ticks delay was not on time ({elapsed} ticks passed instead)");
            }
        }
    };

    macro_rules! cassette {
        ($($x:ident),* $(,)?) => { $(
            // Move the value to ensure that it is owned
            let mut $x = $x;
            // Shadow the original binding so that it can't be directly accessed
            // ever again.
            #[allow(unused_mut)]
            let mut $x = unsafe {
                core::pin::Pin::new_unchecked(&mut $x)
            };

            let mut $x = Cassette::new($x);
        )* }
    }

    let d1 = build_delay_test(Some(100), 100);
    cassette!(d1);

    let d2 = build_delay_test(None, 300);
    cassette!(d2);

    let d3 = build_delay_test(None, 400);
    cassette!(d3);

    macro_rules! poll {
        ($($fut:ident),*) => {
            $(if !$fut.is_done() {
                    $fut.poll_on();
            })*
        };
    }

    // Do an initial poll to set up all of the waiting futures
    poll!(d1, d2, d3);

    for _ in 0..500 {
        // We only poll the waiting futures if an
        // interrupt occured or if an artificial delay
        // has passed.
        if peripheral::tick() {
            poll!(d1, d2, d3);
        }

        if peripheral::now() == 0 {
            // First, we want to be waiting for our 300 tick delay
            assert_eq!(TestMonoBackend::compare(), Some(301));
        }

        if peripheral::now() == 100 {
            // After 100 ticks, we enqueue a new delay that is supposed to last
            // until the 200-tick-mark
            assert_eq!(TestMonoBackend::compare(), Some(201));
        }

        if peripheral::now() == 201 {
            // After 200 ticks, we dequeue the 200-tick-mark delay and
            // requeue the 300 tick delay
            assert_eq!(TestMonoBackend::compare(), Some(301));
        }

        if peripheral::now() == 301 {
            // After 300 ticks, we dequeue the 300-tick-mark delay and
            // go to the 400 tick delay that is already enqueued
            assert_eq!(TestMonoBackend::compare(), Some(401));
        }
    }

    assert!(d1.is_done() && d2.is_done() && d3.is_done());
}
