mod build;
mod command;

use anyhow::bail;
use core::fmt;
use std::{
    error::Error,
    ffi::OsString,
    path::{Path, PathBuf},
    process,
    process::ExitStatus,
    str,
};
use structopt::StructOpt;

use crate::{
    build::{build_hexpath, compare_builds, init_build_dir},
    command::{run_command, run_successful, BuildMode, CargoCommand},
};

const ARMV6M: &str = "thumbv6m-none-eabi";
const ARMV7M: &str = "thumbv7m-none-eabi";

#[derive(Debug, StructOpt)]
struct Options {
    #[structopt(short, long)]
    target: String,
}

#[derive(Debug, Clone)]
pub struct RunResult {
    exit_status: ExitStatus,
    output: String,
}

#[derive(Debug)]
pub enum TestRunError {
    FileCmpError { expected: String, got: String },
    FileError { file: String },
    PathConversionError(OsString),
    CommandError(RunResult),
    IncompatibleCommand,
}

impl fmt::Display for TestRunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestRunError::FileCmpError { expected, got } => {
                writeln!(f, "Differing output in files.")?;
                writeln!(f, "")?;
                writeln!(f, "Expected:")?;
                writeln!(f, "{}", expected)?;
                writeln!(f, "")?;
                writeln!(f, "Got:")?;
                write!(f, "{}", got)
            }
            TestRunError::FileError { file } => {
                write!(f, "File error on: {}", file)
            }
            TestRunError::CommandError(e) => {
                write!(
                    f,
                    "Command failed with exit status {}: {}",
                    e.exit_status, e.output
                )
            }
            TestRunError::PathConversionError(p) => {
                write!(f, "Can't convert path from `OsString` to `String`: {:?}", p)
            }
            TestRunError::IncompatibleCommand => {
                write!(f, "Can't run that command in this context")
            }
        }
    }
}

impl Error for TestRunError {}

fn main() -> anyhow::Result<()> {
    // if there's an `xtask` folder, we're *probably* at the root of this repo (we can't just
    // check the name of `env::current_dir()` because people might clone it into a different name)
    let probably_running_from_repo_root = Path::new("./xtask").exists();
    if probably_running_from_repo_root == false {
        bail!("xtasks can only be executed from the root of the `cortex-m-rtic` repository");
    }

    let targets = [ARMV7M, ARMV6M];

    let examples: Vec<_> = std::fs::read_dir("./examples")?
        .filter_map(|path| {
            path.map(|p| p.path().file_stem().unwrap().to_str().unwrap().to_string())
                .ok()
        })
        .collect();

    // let examples = &[
    //     "idle",
    //     "init",
    //     "hardware",
    //     "preempt",
    //     "binds",
    //     "lock",
    //     "multilock",
    //     "only-shared-access",
    //     "task",
    //     "message",
    //     "capacity",
    //     "not-sync",
    //     "generics",
    //     "pool",
    //     "ramfunc",
    //     "peripherals-taken",
    // ];

    let opts = Options::from_args();
    let target = &opts.target;

    init_build_dir()?;

    if target == "all" {
        for t in targets {
            run_test(t, &examples)?;
            build_test(t, &examples)?;
        }
    } else if targets.contains(&target.as_str()) {
        run_test(&target, &examples)?;
        build_test(&target, &examples)?;
    } else {
        eprintln!(
            "The target you specified is not available. Available targets are:\
                    \n{:?}\n\
                    as well as `all` (testing on all of the above)",
            targets
        );
        process::exit(1);
    }

    Ok(())
}

fn run_test(target: &str, examples: &[String]) -> anyhow::Result<()> {
    for example in examples {
        let cmd = CargoCommand::Run {
            example,
            target,
            features: None,
            mode: BuildMode::Release,
        };

        arm_example(&cmd, 1)?;

        arm_example(
            &CargoCommand::Build {
                example,
                target,
                features: None,
                mode: BuildMode::Release,
            },
            1,
        )?;
    }

    Ok(())
}

// run example binary `example`
fn arm_example(command: &CargoCommand, build_num: u32) -> anyhow::Result<()> {
    match *command {
        CargoCommand::Run {
            example,
            target,
            features,
            mode,
        }
        | CargoCommand::Build {
            example,
            target,
            features,
            mode,
        } => {
            let run_file = format!("{}.run", example);
            let expected_output_file = ["ci", "expected", &run_file]
                .iter()
                .collect::<PathBuf>()
                .into_os_string()
                .into_string()
                .map_err(|e| TestRunError::PathConversionError(e))?;

            // command is either build or run
            let cargo_run_result = run_command(&command)?;
            println!("{}", cargo_run_result.output);

            match &command {
                CargoCommand::Run { .. } => {
                    run_successful(&cargo_run_result, expected_output_file)?;
                }
                _ => (),
            }

            // now, prepare to objcopy
            let hexpath = build_hexpath(example, features, mode, build_num)?;

            run_command(&CargoCommand::Objcopy {
                example,
                target,
                features,
                ihex: &hexpath,
            })?;

            Ok(())
        }
        _ => Err(anyhow::Error::new(TestRunError::IncompatibleCommand)),
    }
}

fn build_test(target: &str, examples: &[String]) -> anyhow::Result<()> {
    run_command(&CargoCommand::Clean)?;

    let mut built = vec![];
    let build_path: PathBuf = ["target", target, "release", "examples"].iter().collect();

    for example in examples {
        let no_features = None;
        arm_example(
            &CargoCommand::Build {
                target,
                example,
                mode: BuildMode::Release,
                features: no_features,
            },
            2,
        )?;
        let expected = build_hexpath(example, no_features, BuildMode::Release, 1)?;
        let got = build_hexpath(example, no_features, BuildMode::Release, 2)?;

        compare_builds(expected, got)?;

        arm_example(
            &CargoCommand::Build {
                target,
                example,
                mode: BuildMode::Release,
                features: no_features,
            },
            2,
        )?;
        let expected = build_hexpath(example, no_features, BuildMode::Release, 1)?;
        let got = build_hexpath(example, no_features, BuildMode::Release, 2)?;

        compare_builds(expected, got)?;

        built.push(build_path.join(example));
    }

    let example_paths: Vec<&Path> = built.iter().map(|p| p.as_path()).collect();
    let size_run_result = run_command(&CargoCommand::Size { example_paths })?;

    if size_run_result.exit_status.success() {
        println!("{}", size_run_result.output);
    }

    Ok(())
}

// /// Check if lines in `output` contain `pattern` and print matching lines
// fn print_from_output(pattern: &str, lines: &str) {
//     let lines = lines.split("\n");
//     for line in lines {
//         if line.contains(pattern) {
//             println!("{}", line);
//         }
//     }
// }
