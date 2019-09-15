use std::path::PathBuf;

use compiletest_rs::{common::Mode, Config};

#[test]
fn ui() {
    let mut config = Config::default();

    config.mode = Mode::Ui;
    config.src_base = PathBuf::from("ui/single");
    config.target_rustcflags = Some(
        "--edition=2018 -L target/debug/deps -Z unstable-options --extern rtfm --extern lm3s6965"
            .to_owned(),
    );
    config.link_deps();
    config.clean_rmeta();

    compiletest_rs::run_tests(&config);
}
