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
    build::init_build_dir,
    command::{run_command, run_successful, BuildMode, CargoCommand},
};

const ARMV6M: &str = "thumbv6m-none-eabi";
const ARMV7M: &str = "thumbv7m-none-eabi";

#[derive(Debug, StructOpt)]
struct Options {
    /// For which ARM target to build: v7 or v6
    ///
    /// The permissible targets are:
    /// * all
    ///
    /// * thumbv6m-none-eabi
    ///
    /// * thumbv7m-none-eabi
    #[structopt(short, long)]
    target: String,
    /// Example to run, by default all examples are run
    ///
    /// Example: `cargo xtask --target <..> --example complex`
    #[structopt(short, long)]
    example: Option<String>,
    /// Enables also running `cargo size` on the selected examples
    ///
    /// To pass options to `cargo size`, add `--` and then the following
    /// arguments will be passed on
    ///
    /// Example: `cargo xtask --target <..> -s -- -A`
    #[structopt(short, long)]
    size: bool,
    /// Options to pass to `cargo size`
    #[structopt(subcommand)]
    sizearguments: Option<Sizearguments>,
}

#[derive(Clone, Debug, PartialEq, StructOpt)]
pub enum Sizearguments {
    // `external_subcommand` tells structopt to put
    // all the extra arguments into this Vec
    #[structopt(external_subcommand)]
    Other(Vec<String>),
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
                writeln!(f, "Differing output in files.\n")?;
                writeln!(f, "Expected:")?;
                writeln!(f, "{expected}\n")?;
                writeln!(f, "Got:")?;
                write!(f, "{got}")
            }
            TestRunError::FileError { file } => {
                write!(f, "File error on: {file}")
            }
            TestRunError::CommandError(e) => {
                write!(
                    f,
                    "Command failed with exit status {}: {}",
                    e.exit_status, e.output
                )
            }
            TestRunError::PathConversionError(p) => {
                write!(f, "Can't convert path from `OsString` to `String`: {p:?}")
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
    if !probably_running_from_repo_root {
        bail!("xtasks can only be executed from the root of the `rtic` repository");
    }

    let targets = [ARMV7M, ARMV6M];

    let mut examples: Vec<_> = std::fs::read_dir("./examples")?
        .filter_map(|p| p.ok())
        .map(|p| p.path())
        .filter(|p| p.display().to_string().ends_with(".rs"))
        .map(|path| path.file_stem().unwrap().to_str().unwrap().to_string())
        .collect();

    println!("examples: {examples:?}");

    let opts = Options::from_args();
    let target = &opts.target;
    let check_size = opts.size;
    let size_arguments = &opts.sizearguments;
    let example = opts.example;

    if let Some(example) = example {
        if examples.contains(&example) {
            println!("\nTesting example: {example}");
            // If we managed to filter, set the examples to test to only this one
            examples = vec![example]
        } else {
            eprintln!(
                "\nThe example you specified is not available. Available examples are:\
                    \n{examples:#?}\n\
             By default all examples are tested.",
            );
            process::exit(1);
        }
    }
    init_build_dir()?;

    if target == "all" {
        for t in targets {
            run_test(t, &examples, check_size, size_arguments)?;
        }
    } else if targets.contains(&target.as_str()) {
        run_test(target, &examples, check_size, size_arguments)?;
    } else {
        eprintln!(
            "The target you specified is not available. Available targets are:\
                    \n{targets:?}\n\
                    as well as `all` (testing on all of the above)",
        );
        process::exit(1);
    }

    Ok(())
}

fn run_test(
    target: &str,
    examples: &[String],
    check_size: bool,
    size_arguments: &Option<Sizearguments>,
) -> anyhow::Result<()> {
    arm_example(&CargoCommand::BuildAll {
        target,
        features: None,
        mode: BuildMode::Release,
    })?;

    for example in examples {
        let cmd = CargoCommand::Run {
            example,
            target,
            features: None,
            mode: BuildMode::Release,
        };

        arm_example(&cmd)?;
    }
    if check_size {
        for example in examples {
            arm_example(&CargoCommand::Size {
                example,
                target,
                features: None,
                mode: BuildMode::Release,
                arguments: size_arguments.clone(),
            })?;
        }
    }

    Ok(())
}

// run example binary `example`
fn arm_example(command: &CargoCommand) -> anyhow::Result<()> {
    match *command {
        CargoCommand::Run { example, .. } => {
            let run_file = format!("{example}.run");
            let expected_output_file = ["ci", "expected", &run_file]
                .iter()
                .collect::<PathBuf>()
                .into_os_string()
                .into_string()
                .map_err(TestRunError::PathConversionError)?;

            // command is either build or run
            let cargo_run_result = run_command(command)?;
            println!("{}", cargo_run_result.output);

            if let CargoCommand::Run { .. } = &command {
                run_successful(&cargo_run_result, expected_output_file)?;
            }

            Ok(())
        }
        CargoCommand::BuildAll { .. } => {
            // command is either build or run
            let cargo_run_result = run_command(command)?;
            println!("{}", cargo_run_result.output);

            Ok(())
        }
        CargoCommand::Size { .. } => {
            let cargo_run_result = run_command(command)?;
            println!("{}", cargo_run_result.output);
            Ok(())
        }
    }
}
