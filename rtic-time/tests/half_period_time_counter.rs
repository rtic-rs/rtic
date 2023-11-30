use atomic_polyfill::{AtomicU16, Ordering};
use rtic_time::half_period_counter::calculate_now;

macro_rules! do_test {
    ($periods:literal, $counter:literal, $expected:literal) => {{
        let periods: AtomicU16 = AtomicU16::new($periods);
        let counter: u8 = $counter;
        let expected: u32 = $expected;
        let actual: u32 = calculate_now(&periods, || counter);
        assert_eq!(
            actual,
            expected,
            "Expected: ({} | {}) => {}, got: {}",
            periods.load(Ordering::Relaxed),
            counter,
            expected,
            actual
        );
    }};
}

#[test]
fn half_period_time_counter() {
    do_test!(0, 0, 0);
    do_test!(0, 1, 1);

    do_test!(0, 126, 126);
    do_test!(1, 126, 382); // This is why it's important to load the periods before the counter
    do_test!(0, 127, 127);
    do_test!(1, 127, 383);
    do_test!(0, 128, 128);
    do_test!(1, 128, 128);
    do_test!(0, 129, 129);
    do_test!(1, 129, 129);

    do_test!(1, 254, 254);
    do_test!(2, 254, 510);
    do_test!(1, 255, 255);
    do_test!(2, 255, 511);
    do_test!(1, 0, 256);
    do_test!(2, 0, 256);
    do_test!(1, 1, 257);
    do_test!(2, 1, 257);

    do_test!(2, 126, 382);
    do_test!(3, 126, 638);
    do_test!(2, 127, 383);
    do_test!(3, 127, 639);
    do_test!(2, 128, 384);
    do_test!(3, 128, 384);
    do_test!(2, 129, 385);
    do_test!(3, 129, 385);
}
