//! A test that verifies the correctness of the [`TimerQueue`].
//!
//! To run this test, you need to activate the `critical-section/std` feature.

use std::{fmt::Debug, time::Duration};

use parking_lot::Mutex;
use rtic_time::{Monotonic, TimerQueue};
use tokio::sync::watch;

static START: Mutex<Option<std::time::Instant>> = Mutex::new(None);
pub struct StdTokioMono;

// An instant that "starts" at Duration::ZERO, so we can
// have a zero value.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct Instant(std::time::Duration);

impl Instant {
    pub fn init() {
        assert!(START.lock().is_none());
        let _ = START.lock().insert(std::time::Instant::now());
    }

    pub fn now() -> Self {
        let start = channel_read("Instant start not initialized", &START);
        Self(start.elapsed())
    }
}

impl core::ops::Add<Duration> for Instant {
    type Output = Instant;

    fn add(self, rhs: Duration) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl core::ops::Sub<Duration> for Instant {
    type Output = Instant;

    fn sub(self, rhs: Duration) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl core::ops::Sub<Instant> for Instant {
    type Output = Duration;

    fn sub(self, rhs: Instant) -> Self::Output {
        self.0 - rhs.0
    }
}

fn channel_read<T: Clone>(msg: &str, channel: &Mutex<Option<T>>) -> T {
    channel.lock().as_ref().expect(msg).clone()
}

fn event_write<T: Debug>(msg: &str, channel: &Mutex<Option<watch::Sender<T>>>, value: T) {
    channel.lock().as_ref().expect(msg).send(value).unwrap()
}

static COMPARE_RX: Mutex<Option<watch::Receiver<Instant>>> = Mutex::new(None);
static COMPARE_TX: Mutex<Option<watch::Sender<Instant>>> = Mutex::new(None);
static INTERRUPT_RX: Mutex<Option<watch::Receiver<()>>> = Mutex::new(None);
static INTERRUPT_TX: Mutex<Option<watch::Sender<()>>> = Mutex::new(None);

impl StdTokioMono {
    /// Initialize the monotonic.
    ///
    /// Returns a [`watch::Sender`] that will cause the interrupt
    /// & compare-change tasks to exit if a value is sent to it or it
    /// is dropped.
    #[must_use = "Dropping the returned Sender stops interrupts & compare-change events"]
    pub fn init() -> watch::Sender<()> {
        Instant::init();
        let (compare_tx, compare_rx) = watch::channel(Instant(Duration::ZERO));
        let (irq_tx, irq_rx) = watch::channel(());

        assert!(COMPARE_RX.lock().is_none());
        assert!(COMPARE_TX.lock().is_none());
        let _ = COMPARE_RX.lock().insert(compare_rx);
        let _ = COMPARE_TX.lock().insert(compare_tx);

        assert!(INTERRUPT_RX.lock().is_none());
        assert!(INTERRUPT_TX.lock().is_none());
        let _ = INTERRUPT_RX.lock().insert(irq_rx);
        let _ = INTERRUPT_TX.lock().insert(irq_tx);

        Self::queue().initialize(Self);

        let (killer_tx, mut killer_rx) = watch::channel(());

        let mut killer_clone = killer_rx.clone();
        // Set up a task that watches for changes to the COMPARE value,
        // and re-starts a timeout based on that change
        tokio::spawn(async move {
            let mut compare_rx = channel_read("Compare RX not initialized", &COMPARE_RX);

            loop {
                let compare = compare_rx.borrow().clone();

                let end = channel_read("Start not initialized", &START) + compare.0;

                tokio::select! {
                    _ = killer_clone.changed() => break,
                    _ = compare_rx.changed() => {},
                    _ = tokio::time::sleep_until(end.into()) => {
                        event_write("Interrupt TX not initialized", &INTERRUPT_TX, ());
                        // Sleep for a bit to avoid re-firing the interrupt a bunch of
                        // times.
                        tokio::time::sleep(Duration::from_millis(1)).await;
                    },
                }
            }
        });

        // Set up a task that emulates an interrupt handler, calling `on_monotonic_interrupt`
        // whenever an "interrupt" is generated.
        tokio::spawn(async move {
            let mut interrupt_rx = channel_read("Interrupt RX not initialized.", &INTERRUPT_RX);

            loop {
                tokio::select! {
                    _ = killer_rx.changed() => break,
                    _ = interrupt_rx.changed() => {
                        // TODO: verify that we get interrupts triggered by an
                        // explicit pend or due to COMPARE at the correct time.
                    }
                }

                unsafe {
                    StdTokioMono::queue().on_monotonic_interrupt();
                }
            }
        });

        killer_tx
    }

    /// Used to access the underlying timer queue
    pub fn queue() -> &'static TimerQueue<StdTokioMono> {
        &TIMER_QUEUE
    }
}

impl Monotonic for StdTokioMono {
    const ZERO: Self::Instant = Instant(Duration::ZERO);

    type Instant = Instant;

    type Duration = Duration;

    fn now() -> Self::Instant {
        Instant::now()
    }

    fn set_compare(instant: Self::Instant) {
        // TODO: verify that we receive the correct amount & values
        // for `set_compare`.

        log::info!("Setting compare to {} ms", instant.0.as_millis());

        event_write("Compare TX not initialized", &COMPARE_TX, instant);
    }

    fn clear_compare_flag() {}

    fn pend_interrupt() {
        event_write("Interrupt TX not initialized", &INTERRUPT_TX, ());
    }
}

static TIMER_QUEUE: TimerQueue<StdTokioMono> = TimerQueue::new();

#[tokio::test]
async fn main() {
    pretty_env_logger::init();

    let _interrupt_killer = StdTokioMono::init();

    let start = std::time::Instant::now();

    let build_delay_test = |threshold: u128, pre_delay: Option<u64>, delay: u64| {
        let delay = Duration::from_millis(delay);
        let pre_delay = pre_delay.map(Duration::from_millis);

        let total = if let Some(pre_delay) = pre_delay {
            pre_delay + delay
        } else {
            delay
        };
        let total_millis = total.as_millis();
        async move {
            if let Some(pre_delay) = pre_delay {
                tokio::time::sleep_until((start + pre_delay).into()).await;
            }

            StdTokioMono::queue().delay(delay).await;

            let elapsed = start.elapsed().as_millis();
            log::info!("{total_millis} ms delay reached (after {elapsed} ms)");

            if elapsed > total_millis.saturating_add(threshold)
                || elapsed < total_millis.saturating_sub(threshold)
            {
                panic!("{total_millis} ms delay was not on time ({elapsed} ms passed instead)");
            }
        }
    };

    // TODO: depending on the precision of the delays that can be used, this threshold
    // may have to be altered a bit.
    const TIME_THRESHOLD_MS: u128 = 5;

    let sec1 = build_delay_test(TIME_THRESHOLD_MS, Some(100), 100);
    let sec2 = build_delay_test(TIME_THRESHOLD_MS, None, 300);
    let sec3 = build_delay_test(TIME_THRESHOLD_MS, None, 400);

    tokio::join!(sec2, sec1, sec3);
}
