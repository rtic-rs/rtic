extern crate compiletest_rs as compiletest;

use std::path::PathBuf;

use compiletest::common::Mode;

#[test]
fn cfail() {
    let mut config = compiletest::default_config();
    config.mode = Mode::CompileFail;
    config.src_base = PathBuf::from(format!("tests/cfail"));
    config.target_rustcflags =
        Some("-L target/debug -L target/debug/deps ".to_string());

    compiletest::run_tests(&config);
}
