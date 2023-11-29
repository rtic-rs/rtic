//! A test that verifies the sub-tick correctness of the [`TimerQueue`]'s `delay` functionality.
//!
//! To run this test, you need to activate the `critical-section/std` feature.

use std::{
    fmt::Debug,
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Arc,
    },
    task::Context,
};

use ::fugit::ExtU64Ceil;
use cooked_waker::{IntoWaker, WakeRef};
use parking_lot::Mutex;
use rtic_time::{Monotonic, TimeoutError, TimerQueue};

const SUBTICKS_PER_TICK: u32 = 10;
struct SubtickTestTimer;
static TIMER_QUEUE: TimerQueue<SubtickTestTimer> = TimerQueue::new();
static NOW_SUBTICKS: AtomicU64 = AtomicU64::new(0);
static COMPARE_TICKS: Mutex<Option<u64>> = Mutex::new(None);

impl Monotonic for SubtickTestTimer {
    const ZERO: Self::Instant = Self::Instant::from_ticks(0);
    const TICK_PERIOD: Self::Duration = Self::Duration::from_ticks(1);

    type Instant = fugit::Instant<u64, SUBTICKS_PER_TICK, 1000>;
    type Duration = fugit::Duration<u64, SUBTICKS_PER_TICK, 1000>;

    fn now() -> Self::Instant {
        Self::Instant::from_ticks(
            NOW_SUBTICKS.load(Ordering::Relaxed) / u64::from(SUBTICKS_PER_TICK),
        )
    }

    fn set_compare(instant: Self::Instant) {
        *COMPARE_TICKS.lock() = Some(instant.ticks());
    }

    fn clear_compare_flag() {}

    fn pend_interrupt() {
        unsafe {
            Self::__tq().on_monotonic_interrupt();
        }
    }
}

impl SubtickTestTimer {
    pub fn init() {
        Self::__tq().initialize(Self)
    }

    pub fn tick() -> u64 {
        let now = NOW_SUBTICKS.fetch_add(1, Ordering::Relaxed) + 1;
        let ticks = now / u64::from(SUBTICKS_PER_TICK);
        let subticks = now % u64::from(SUBTICKS_PER_TICK);

        let compare = COMPARE_TICKS.lock();

        println!(
            "ticks: {ticks}, subticks: {subticks}, compare: {:?}",
            *compare
        );
        if subticks == 0 && Some(ticks) == *compare {
            unsafe {
                Self::__tq().on_monotonic_interrupt();
            }
        }

        subticks
    }

    pub fn forward_to_subtick(subtick: u64) {
        assert!(subtick < u64::from(SUBTICKS_PER_TICK));
        while Self::tick() != subtick {}
    }

    fn __tq() -> &'static TimerQueue<Self> {
        &TIMER_QUEUE
    }

    /// Delay for some duration of time.
    #[inline]
    pub async fn delay(duration: <Self as Monotonic>::Duration) {
        Self::__tq().delay(duration).await;
    }

    /// Timeout after a specific duration.
    #[inline]
    pub async fn timeout_after<F: core::future::Future>(
        duration: <Self as Monotonic>::Duration,
        future: F,
    ) -> Result<F::Output, TimeoutError> {
        Self::__tq().timeout_after(duration, future).await
    }
}

rtic_time::embedded_hal_delay_impl_fugit64!(SubtickTestTimer);
rtic_time::embedded_hal_async_delay_impl_fugit64!(SubtickTestTimer);

// A simple struct that counts the number of times it is awoken. Can't
// be awoken by value (because that would discard the counter), so we
// must instead wrap it in an Arc.
#[derive(Debug, Default)]
struct WakeCounter {
    count: AtomicUsize,
}

impl WakeCounter {
    fn get(&self) -> usize {
        self.count.load(Ordering::SeqCst)
    }
}

impl WakeRef for WakeCounter {
    fn wake_by_ref(&self) {
        let _prev = self.count.fetch_add(1, Ordering::SeqCst);
    }
}

macro_rules! subtick_test {
    (@run $start:expr, $actual_duration:expr, $delay_fn:expr) => {{
        // forward clock to $start
        SubtickTestTimer::forward_to_subtick($start);

        // call wait function
        let delay_fn = $delay_fn;
        let mut future = std::pin::pin!(delay_fn);

        let wakecounter = Arc::new(WakeCounter::default());
        let waker = Arc::clone(&wakecounter).into_waker();
        let mut context = Context::from_waker(&waker);

        let mut finished_after: Option<u64> = None;
        for i in 0..10 * u64::from(SUBTICKS_PER_TICK) {
            if Future::poll(Pin::new(&mut future), &mut context).is_ready() {
                if finished_after.is_none() {
                    finished_after = Some(i);
                }
                break;
            };

            assert_eq!(wakecounter.get(), 0);
            SubtickTestTimer::tick();
        }

        let expected_wakeups = {
            if $actual_duration == 0 {
                0
            } else {
                1
            }
        };
        assert_eq!(wakecounter.get(), expected_wakeups);

        // Tick again to test that we don't get a second wake
        SubtickTestTimer::tick();
        assert_eq!(wakecounter.get(), expected_wakeups);

        println!("Wake Counter: {}", wakecounter.get());
        assert_eq!(
            Some($actual_duration),
            finished_after,
            "Expected to wait {} ticks, but waited {:?} ticks.",
            $actual_duration,
            finished_after,
        );
    }};
    ($start:expr, $min_duration:expr, $actual_duration:expr) => {{
        subtick_test!(@run $start, $actual_duration, async {
            let mut timer = SubtickTestTimer;
            embedded_hal_async::delay::DelayNs::delay_ms(&mut timer, $min_duration).await;
        });
        subtick_test!(@run $start, $actual_duration, async {
            let mut timer = SubtickTestTimer;
            embedded_hal_async::delay::DelayNs::delay_us(&mut timer, 1000 * $min_duration).await;
        });
        subtick_test!(@run $start, $actual_duration, async {
            let mut timer = SubtickTestTimer;
            embedded_hal_async::delay::DelayNs::delay_ns(&mut timer, 1000000 * $min_duration).await;
        });
        subtick_test!(@run $start, $actual_duration, async {
            SubtickTestTimer::delay($min_duration.millis_at_least()).await;
        });
        subtick_test!(@run $start, $actual_duration, async {
            let _ = SubtickTestTimer::timeout_after($min_duration.millis_at_least(), std::future::pending::<()>()).await;
        });
    }};
}

#[test]
fn timer_queue_subtick_precision() {
    SubtickTestTimer::init();

    // subtick_test!(a, b, c) tests the following thing:
    //
    // If we start at subtick a and we need to wait b subticks,
    // then we will actually wait c subticks.
    // The important part is that c is never smaller than b,
    // in all cases, as that would violate the contract of
    // embedded-hal's DelayUs.

    subtick_test!(0, 0, 0);
    subtick_test!(0, 1, 20);
    subtick_test!(0, 10, 20);
    subtick_test!(0, 11, 30);
    subtick_test!(0, 12, 30);

    subtick_test!(1, 0, 0);
    subtick_test!(1, 1, 19);
    subtick_test!(1, 10, 19);
    subtick_test!(1, 11, 29);
    subtick_test!(1, 12, 29);

    subtick_test!(2, 0, 0);
    subtick_test!(2, 1, 18);
    subtick_test!(2, 10, 18);
    subtick_test!(2, 11, 28);
    subtick_test!(2, 12, 28);

    subtick_test!(3, 0, 0);
    subtick_test!(3, 1, 17);
    subtick_test!(3, 10, 17);
    subtick_test!(3, 11, 27);
    subtick_test!(3, 12, 27);

    subtick_test!(8, 0, 0);
    subtick_test!(8, 1, 12);
    subtick_test!(8, 10, 12);
    subtick_test!(8, 11, 22);
    subtick_test!(8, 12, 22);

    subtick_test!(9, 0, 0);
    subtick_test!(9, 1, 11);
    subtick_test!(9, 10, 11);
    subtick_test!(9, 11, 21);
    subtick_test!(9, 12, 21);
}
