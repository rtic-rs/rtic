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

#[derive(Debug)]
pub struct RunResult {
    exit_status: ExitStatus,
    output: String,
}

#[derive(Debug)]
enum TestRunError {
    FileCmpError { file_1: String, file_2: String },
    PathConversionError(OsString),
    CommandError(RunResult),
    IncompatibleCommand,
}

impl fmt::Display for TestRunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestRunError::FileCmpError { file_1, file_2 } => {
                write!(f, "Differing output in Files: {} {}", file_1, file_2)
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
    let examples = &[
        "idle",
        "init",
        "hardware",
        "preempt",
        "binds",
        "resource",
        "lock",
        "multilock",
        "only-shared-access",
        "task",
        "message",
        "capacity",
        "not-sync",
        "generics",
        "pool",
        "ramfunc",
        "peripherals-taken",
    ];

    let opts = Options::from_args();
    let target = &opts.target;

    init_build_dir()?;

    if target == "all" {
        for t in targets {
            run_test(t, examples)?;
            build_test(t, examples)?;
        }
    } else if targets.contains(&target.as_str()) {
        run_test(&target, examples)?;
        build_test(&target, examples)?;
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

fn run_test(target: &str, examples: &[&str]) -> anyhow::Result<()> {
    for example in examples {
        match *example {
            "pool" => {
                if target != ARMV6M {
                    // check this one manually because addresses printed in `pool.run` may vary
                    let features_v7 = Some("__v7");

                    let debug_run_result = run_command(&CargoCommand::Run {
                        example,
                        target,
                        features: features_v7,
                        mode: BuildMode::Debug,
                    })?;

                    if debug_run_result.exit_status.success() {
                        print_from_output("foo(0x2", &debug_run_result.output);
                        print_from_output("bar(0x2", &debug_run_result.output);
                    }

                    let hexpath = &build_hexpath(*example, features_v7, BuildMode::Debug, 1)?;

                    run_command(&CargoCommand::Objcopy {
                        example,
                        target,
                        features: features_v7,
                        ihex: hexpath,
                    })?;

                    let release_run_result = run_command(&CargoCommand::Run {
                        example,
                        target,
                        features: features_v7,
                        mode: BuildMode::Release,
                    })?;

                    if release_run_result.exit_status.success() {
                        print_from_output("foo(0x2", &release_run_result.output);
                        print_from_output("bar(0x2", &release_run_result.output);
                    }

                    let hexpath = &build_hexpath(*example, features_v7, BuildMode::Release, 1)?;
                    run_command(&CargoCommand::Objcopy {
                        example,
                        target,
                        features: features_v7,
                        ihex: hexpath,
                    })?;
                }
            }
            "types" => {
                let features_v7 = Some("__v7");

                // TODO this example doesn't exist anymore, can we remove this case?
                if target != ARMV6M {
                    arm_example(
                        &CargoCommand::Run {
                            example,
                            target,
                            features: features_v7,
                            mode: BuildMode::Debug,
                        },
                        1,
                    )?;
                    arm_example(
                        &CargoCommand::Run {
                            example,
                            target,
                            features: features_v7,
                            mode: BuildMode::Release,
                        },
                        1,
                    )?;
                }
            }
            _ => {
                arm_example(
                    &CargoCommand::Run {
                        example,
                        target,
                        features: None,
                        mode: BuildMode::Debug,
                    },
                    1,
                )?;

                if *example == "types" {
                    arm_example(
                        &CargoCommand::Run {
                            example,
                            target,
                            features: None,
                            mode: BuildMode::Release,
                        },
                        1,
                    )?;
                } else {
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
            }
        }
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
                    if run_successful(&cargo_run_result, expected_output_file).is_err() {
                        return Err(anyhow::Error::new(TestRunError::CommandError(
                            cargo_run_result,
                        )));
                    }
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

fn build_test(target: &str, examples: &[&str]) -> anyhow::Result<()> {
    run_command(&CargoCommand::Clean)?;

    let mut built = vec![];
    let build_path: PathBuf = ["target", target, "debug", "examples"].iter().collect();

    for example in examples {
        match *example {
            "pool" | "types" => {
                if target != ARMV6M {
                    let features_v7 = Some("__v7");

                    arm_example(
                        &CargoCommand::Build {
                            target,
                            example,
                            mode: BuildMode::Debug,
                            features: features_v7,
                        },
                        2,
                    )?;
                    let file_1 = build_hexpath(example, features_v7, BuildMode::Debug, 1)?;
                    let file_2 = build_hexpath(example, features_v7, BuildMode::Debug, 2)?;

                    compare_builds(file_1, file_2)?;

                    arm_example(
                        &CargoCommand::Build {
                            target,
                            example,
                            mode: BuildMode::Release,
                            features: features_v7,
                        },
                        2,
                    )?;
                    let file_1 = build_hexpath(example, features_v7, BuildMode::Release, 1)?;
                    let file_2 = build_hexpath(example, features_v7, BuildMode::Release, 2)?;

                    compare_builds(file_1, file_2)?;

                    built.push(build_path.join(example));
                }
            }
            _ => {
                let no_features = None;
                arm_example(
                    &CargoCommand::Build {
                        target,
                        example,
                        mode: BuildMode::Debug,
                        features: no_features,
                    },
                    2,
                )?;
                let file_1 = build_hexpath(example, no_features, BuildMode::Debug, 1)?;
                let file_2 = build_hexpath(example, no_features, BuildMode::Debug, 2)?;

                compare_builds(file_1, file_2)?;

                arm_example(
                    &CargoCommand::Build {
                        target,
                        example,
                        mode: BuildMode::Release,
                        features: no_features,
                    },
                    2,
                )?;
                let file_1 = build_hexpath(example, no_features, BuildMode::Release, 1)?;
                let file_2 = build_hexpath(example, no_features, BuildMode::Release, 2)?;

                compare_builds(file_1, file_2)?;

                built.push(build_path.join(example));
            }
        }
    }

    let example_paths: Vec<&Path> = built.iter().map(|p| p.as_path()).collect();
    let size_run_result = run_command(&CargoCommand::Size { example_paths })?;

    if size_run_result.exit_status.success() {
        println!("{}", size_run_result.output);
    }

    Ok(())
}

/// Check if lines in `output` contain `pattern` and print matching lines
fn print_from_output(pattern: &str, lines: &str) {
    let lines = lines.split("\n");
    for line in lines {
        if line.contains(pattern) {
            println!("{}", line);
        }
    }
}
