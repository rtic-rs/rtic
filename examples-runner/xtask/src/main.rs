mod build;
mod command;

use anyhow::bail;
use clap::Parser;
use core::fmt;
use std::{
    error::Error,
    ffi::OsString,
    path::{Path, PathBuf},
    process,
    process::ExitStatus,
    str,
};

use crate::{
    build::init_build_dir,
    command::{run_command, run_successful, BuildMode, CargoCommand, Runner},
};

const ARMV6M: &str = "thumbv6m-none-eabi";
const ARMV7M: &str = "thumbv7m-none-eabi";

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Options {
    #[clap(short, long)]
    target: String,
    #[clap(short, long, arg_enum)]
    runner: Runner,
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

    let examples: Vec<_> = std::fs::read_dir("./src/bin")?
        .filter_map(|path| {
            path.map(|p| p.path().file_stem().unwrap().to_str().unwrap().to_string())
                .ok()
        })
        .collect();

    let opts = Options::parse();
    let target = &opts.target;

    init_build_dir()?;

    if target == "all" {
        for t in targets {
            run_tests(t, opts.runner, &examples)?;
        }
    } else if targets.contains(&target.as_str()) {
        run_tests(&target, opts.runner, &examples)?;
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

fn run_tests(target: &str, runner: Runner, examples: &[String]) -> anyhow::Result<()> {
    let features = Some(match runner {
        Runner::Qemu => "qemu",
        Runner::EmbeddedCi => "embedded-ci",
    });

    arm_example(&CargoCommand::BuildAll {
        target,
        features,
        mode: BuildMode::Release,
    })?;

    for example in examples {
        let cmd = CargoCommand::Run {
            example,
            target,
            features,
            mode: BuildMode::Release,
            runner,
        };

        arm_example(&cmd)?;
    }

    Ok(())
}

// run example binary `example`
fn arm_example(command: &CargoCommand) -> anyhow::Result<()> {
    match *command {
        CargoCommand::Run { example, .. } => {
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

            Ok(())
        }
        CargoCommand::BuildAll { .. } => {
            // command is either build or run
            let cargo_run_result = run_command(&command)?;
            println!("{}", cargo_run_result.output);

            Ok(())
        } // _ => Err(anyhow::Error::new(TestRunError::IncompatibleCommand)),
    }
}
