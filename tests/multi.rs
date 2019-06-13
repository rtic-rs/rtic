use std::path::PathBuf;

use compiletest_rs::{common::Mode, Config};

#[test]
fn ui() {
    let mut config = Config::default();

    config.mode = Mode::Ui;
    config.src_base = PathBuf::from("ui/multi");
    config.target_rustcflags = Some("--edition=2018 -Z unstable-options --extern rtfm".to_owned());
    config.link_deps();
    config.clean_rmeta();

    compiletest_rs::run_tests(&config);
}
