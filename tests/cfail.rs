extern crate compiletest_rs as compiletest;

use std::path::PathBuf;

use compiletest::common::Mode;
use compiletest::Config;

#[test]
fn cfail() {
    let mut config = Config::default();
    config.mode = Mode::CompileFail;
    config.src_base = PathBuf::from(format!("tests/cfail"));
    config.target = "x86_64-unknown-linux-gnu".to_owned();
    config.target_rustcflags = Some(
        "-C panic=abort \
         -L target/debug/deps \
         -L target/x86_64-unknown-linux-gnu/debug \
         -L target/x86_64-unknown-linux-gnu/debug/deps "
            .to_string(),
    );

    compiletest::run_tests(&config);
}
