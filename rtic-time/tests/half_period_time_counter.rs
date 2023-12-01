use core::sync::atomic::{AtomicU16, AtomicU32, Ordering};
use rtic_time::half_period_counter::calculate_now;

macro_rules! do_test_u8 {
    ($periods:literal, $counter:literal, $expected:literal) => {{
        let periods: AtomicU32 = AtomicU32::new($periods);
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

macro_rules! do_test_u16 {
    ($periods:literal, $counter:literal, $expected:literal) => {{
        let periods: AtomicU16 = AtomicU16::new($periods);
        let counter: u16 = $counter;
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
fn half_period_time_counter_u8() {
    do_test_u8!(0, 0, 0);
    do_test_u8!(0, 1, 1);

    do_test_u8!(0, 126, 126);
    do_test_u8!(1, 126, 382); // This is why it's important to load the periods before the counter
    do_test_u8!(0, 127, 127);
    do_test_u8!(1, 127, 383);
    do_test_u8!(0, 128, 128);
    do_test_u8!(1, 128, 128);
    do_test_u8!(0, 129, 129);
    do_test_u8!(1, 129, 129);

    do_test_u8!(1, 254, 254);
    do_test_u8!(2, 254, 510);
    do_test_u8!(1, 255, 255);
    do_test_u8!(2, 255, 511);
    do_test_u8!(1, 0, 256);
    do_test_u8!(2, 0, 256);
    do_test_u8!(1, 1, 257);
    do_test_u8!(2, 1, 257);

    do_test_u8!(2, 126, 382);
    do_test_u8!(3, 126, 638);
    do_test_u8!(2, 127, 383);
    do_test_u8!(3, 127, 639);
    do_test_u8!(2, 128, 384);
    do_test_u8!(3, 128, 384);
    do_test_u8!(2, 129, 385);
    do_test_u8!(3, 129, 385);
}

#[test]
fn half_period_time_counter_u16() {
    do_test_u16!(0, 0, 0);
    do_test_u16!(0, 1, 1);

    do_test_u16!(0, 32766, 32766);
    do_test_u16!(1, 32766, 98302); // This is why it's important to load the periods before the counter
    do_test_u16!(0, 32767, 32767);
    do_test_u16!(1, 32767, 98303);
    do_test_u16!(0, 32768, 32768);
    do_test_u16!(1, 32768, 32768);
    do_test_u16!(0, 32769, 32769);
    do_test_u16!(1, 32769, 32769);

    do_test_u16!(1, 65534, 65534);
    do_test_u16!(2, 65534, 131070);
    do_test_u16!(1, 65535, 65535);
    do_test_u16!(2, 65535, 131071);
    do_test_u16!(1, 0, 65536);
    do_test_u16!(2, 0, 65536);
    do_test_u16!(1, 1, 65537);
    do_test_u16!(2, 1, 65537);

    do_test_u16!(2, 32766, 98302);
    do_test_u16!(3, 32766, 163838);
    do_test_u16!(2, 32767, 98303);
    do_test_u16!(3, 32767, 163839);
    do_test_u16!(2, 32768, 98304);
    do_test_u16!(3, 32768, 98304);
    do_test_u16!(2, 32769, 98305);
    do_test_u16!(3, 32769, 98305);
}
