use std::{fs, path::PathBuf, process::Command};

use compiletest_rs::{common::Mode, Config};
use tempdir::TempDir;

#[test]
fn cfail() {
    let mut config = Config::default();

    config.mode = Mode::CompileFail;
    config.src_base = PathBuf::from("tests/cfail");
    config.link_deps();

    // remove duplicate and trailing `-L` flags
    let mut s = String::new();
    if let Some(flags) = config.target_rustcflags.as_mut() {
        let mut iter = flags.split_whitespace().peekable();

        while let Some(flag) = iter.next() {
            if flag == "-L" && (iter.peek() == Some(&"-L") || iter.peek() == None) {
                iter.next();
                continue;
            }

            s += flag;
            s += " ";
        }

        // path to proc-macro crate
        s += "-L target/debug/deps ";

        // avoid "error: language item required, but not found: `eh_personality`"
        s += "-C panic=abort ";
    }

    let td = TempDir::new("rtfm").unwrap();
    for f in fs::read_dir("tests/cpass").unwrap() {
        let f = f.unwrap().path();
        let name = f.file_stem().unwrap().to_str().unwrap();

        assert!(
            Command::new("rustc")
                .args(s.split_whitespace())
                .arg(f.display().to_string())
                .arg("-o")
                .arg(td.path().join(name).display().to_string())
                .arg("-C")
                .arg("linker=true")
                .status()
                .unwrap()
                .success()
        );
    }

    config.target_rustcflags = Some(s);
    config.clean_rmeta();

    compiletest_rs::run_tests(&config);
}
