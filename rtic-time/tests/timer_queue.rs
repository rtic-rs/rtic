//! A test that verifies the correctness of the [`TimerQueue`].
//!
//! To run this test, you need to activate the `critical-section/std` feature.

use std::{
    fmt::Debug,
    task::{Poll, Waker},
};

use cassette::Cassette;
use parking_lot::Mutex;
use rtic_time::{Monotonic, TimerQueue};

static NOW: Mutex<Option<Instant>> = Mutex::new(None);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct Duration(u64);

impl Duration {
    pub fn from_ticks(millis: u64) -> Self {
        Self(millis)
    }

    pub fn as_ticks(&self) -> u64 {
        self.0
    }
}

impl core::ops::Add<Duration> for Duration {
    type Output = Duration;

    fn add(self, rhs: Duration) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl From<Duration> for Instant {
    fn from(value: Duration) -> Self {
        Instant(value.0)
    }
}

static WAKERS: Mutex<Vec<Waker>> = Mutex::new(Vec::new());

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct Instant(u64);

impl Instant {
    const ZERO: Self = Self(0);

    pub fn tick() -> bool {
        // If we've never ticked before, initialize the clock.
        if NOW.lock().is_none() {
            *NOW.lock() = Some(Instant::ZERO);
        }
        // We've ticked before, add one to the clock
        else {
            let now = Instant::now();
            let new_time = now + Duration(1);
            *NOW.lock() = Some(new_time);
        }

        let had_wakers = !WAKERS.lock().is_empty();
        // Wake up all things waiting for a specific time to happen.
        for waker in WAKERS.lock().drain(..) {
            waker.wake_by_ref();
        }

        let had_interrupt = TestMono::tick(false);

        had_interrupt || had_wakers
    }

    pub fn now() -> Self {
        NOW.lock().clone().unwrap_or(Instant::ZERO)
    }

    pub fn elapsed(&self) -> Duration {
        Duration(Self::now().0 - self.0)
    }

    pub async fn wait_until(time: Instant) {
        core::future::poll_fn(|ctx| {
            if Instant::now() >= time {
                Poll::Ready(())
            } else {
                WAKERS.lock().push(ctx.waker().clone());
                Poll::Pending
            }
        })
        .await;
    }
}

impl From<u64> for Instant {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl core::ops::Add<Duration> for Instant {
    type Output = Instant;

    fn add(self, rhs: Duration) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl core::ops::Sub<Duration> for Instant {
    type Output = Instant;

    fn sub(self, rhs: Duration) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl core::ops::Sub<Instant> for Instant {
    type Output = Duration;

    fn sub(self, rhs: Instant) -> Self::Output {
        Duration(self.0 - rhs.0)
    }
}

static COMPARE: Mutex<Option<Instant>> = Mutex::new(None);
static TIMER_QUEUE: TimerQueue<TestMono> = TimerQueue::new();

pub struct TestMono;

impl TestMono {
    pub fn tick(force_interrupt: bool) -> bool {
        let now = Instant::now();

        let compare_reached = Some(now) == Self::compare();
        let interrupt = compare_reached || force_interrupt;

        if interrupt {
            unsafe {
                TestMono::queue().on_monotonic_interrupt();
            }
            true
        } else {
            false
        }
    }

    /// Initialize the monotonic.
    pub fn init() {
        Self::queue().initialize(Self);
    }

    /// Used to access the underlying timer queue
    pub fn queue() -> &'static TimerQueue<TestMono> {
        &TIMER_QUEUE
    }

    pub fn compare() -> Option<Instant> {
        COMPARE.lock().clone()
    }
}

impl Monotonic for TestMono {
    const ZERO: Self::Instant = Instant::ZERO;

    type Instant = Instant;

    type Duration = Duration;

    fn now() -> Self::Instant {
        Instant::now()
    }

    fn set_compare(instant: Self::Instant) {
        let _ = COMPARE.lock().insert(instant);
    }

    fn clear_compare_flag() {}

    fn pend_interrupt() {
        Self::tick(true);
    }
}

#[test]
fn timer_queue() {
    TestMono::init();
    let start = Instant::ZERO;

    let build_delay_test = |pre_delay: Option<u64>, delay: u64| {
        let delay = Duration::from_ticks(delay);
        let pre_delay = pre_delay.map(Duration::from_ticks);

        let total = if let Some(pre_delay) = pre_delay {
            pre_delay + delay
        } else {
            delay
        };
        let total_millis = total.as_ticks();

        async move {
            // A `pre_delay` simulates a delay in scheduling,
            // without the `pre_delay` being present in the timer
            // queue
            if let Some(pre_delay) = pre_delay {
                Instant::wait_until(start + pre_delay).await;
            }

            TestMono::queue().delay(delay).await;

            let elapsed = start.elapsed().as_ticks();
            println!("{total_millis} ticks delay reached after {elapsed} ticks");

            if elapsed != total_millis {
                panic!(
                    "{total_millis} ticks delay was not on time ({elapsed} ticks passed instead)"
                );
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
        if Instant::tick() {
            poll!(d1, d2, d3);
        }

        if Instant::now() == 0.into() {
            // First, we want to be waiting for our 300 tick delay
            assert_eq!(TestMono::compare(), Some(300.into()));
        }

        if Instant::now() == 100.into() {
            // After 100 ticks, we enqueue a new delay that is supposed to last
            // until the 200-tick-mark
            assert_eq!(TestMono::compare(), Some(200.into()));
        }

        if Instant::now() == 200.into() {
            // After 200 ticks, we dequeue the 200-tick-mark delay and
            // requeue the 300 tick delay
            assert_eq!(TestMono::compare(), Some(300.into()));
        }

        if Instant::now() == 300.into() {
            // After 300 ticks, we dequeue the 300-tick-mark delay and
            // go to the 400 tick delay that is already enqueued
            assert_eq!(TestMono::compare(), Some(400.into()));
        }
    }

    assert!(d1.is_done() && d2.is_done() && d3.is_done());
}
