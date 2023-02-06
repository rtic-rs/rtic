mod build;
mod command;

use anyhow::bail;
use clap::{Parser, Subcommand};
use core::fmt;
use rayon::prelude::*;
use std::{
    error::Error,
    ffi::OsString,
    fs::File,
    io::prelude::*,
    path::{Path, PathBuf},
    process,
    process::ExitStatus,
    str,
};

use env_logger::Env;
use log::{debug, error, info, log_enabled, trace, Level};

use crate::{
    build::init_build_dir,
    command::{run_command, run_successful, BuildMode, CargoCommand},
};

const ARMV6M: &str = "thumbv6m-none-eabi";
const ARMV7M: &str = "thumbv7m-none-eabi";
const ARMV8MBASE: &str = "thumbv8m.base-none-eabi";
const ARMV8MMAIN: &str = "thumbv8m.main-none-eabi";

const DEFAULT_FEATURES: Option<&str> = Some("test-critical-section");

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
/// RTIC xtask powered testing toolbox
struct Cli {
    /// For which ARM target to build: v7 or v6
    ///
    /// Defaults to all targets if omitted.
    /// The permissible targets are:
    ///
    /// thumbv6m-none-eabi
    /// thumbv7m-none-eabi
    /// thumbv8m.base-none-eabi
    /// thumbv8m.main-none-eabi
    #[arg(short, long)]
    target: Option<String>,

    /// List of comma separated examples to run, all others are excluded
    ///
    /// If omitted all examples are run
    ///
    /// Example: `cargo xtask --example complex,spawn,init`
    /// would include complex, spawn and init
    #[arg(short, long, group = "example_group")]
    example: Option<String>,

    /// List of comma separated examples to exclude, all others are included
    ///
    /// If omitted all examples are run
    ///
    /// Example: `cargo xtask --excludeexample complex,spawn,init`
    /// would exclude complex, spawn and init
    #[arg(long, group = "example_group")]
    exampleexclude: Option<String>,

    /// Enable more verbose output, repeat up to `-vvv` for even more
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Subcommand picking which kind of operation
    ///
    /// If omitted run all tests
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run `cargo size` on selected or all examples
    ///
    /// To pass options to `cargo size`, add `--` and then the following
    /// arguments will be passed on
    ///
    /// Example: `cargo xtask size -- -A`
    Size(Size),

    /// Run examples in QEMU and compare against expected output
    ///
    /// Example runtime output is matched against `rtic/ci/expected/`
    Qemu {
        /// If expected output is missing or mismatching, recreate the file
        ///
        /// This overwrites only missing or mismatching
        #[arg(long)]
        overwrite_expected: bool,
    },
    /// Build all examples
    Build,
    /// Check all examples
    Check,
}

#[derive(Debug, Parser)]
struct Size {
    /// Options to pass to `cargo size`
    #[command(subcommand)]
    sizearguments: Option<Sizearguments>,
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub enum Sizearguments {
    /// All remaining flags and options
    #[command(external_subcommand)]
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
use diffy::{create_patch, PatchFormatter};

impl fmt::Display for TestRunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestRunError::FileCmpError { expected, got } => {
                let patch = create_patch(expected, got);
                writeln!(f, "Differing output in files.\n")?;
                let pf = PatchFormatter::new().with_color();
                writeln!(f, "{}", pf.fmt_patch(&patch))?;
                write!(
                    f,
                    "See flag --overwrite-expected to create/update expected output."
                )
            }
            TestRunError::FileError { file } => {
                write!(f, "File error on: {file}\nSee flag --overwrite-expected to create/update expected output.")
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
    for entry in std::fs::read_dir(".").unwrap() {


    let mut targets: Vec<String> = [
        ARMV7M.to_owned(),
        ARMV6M.to_owned(),
        ARMV8MBASE.to_owned(),
        ARMV8MMAIN.to_owned(),
    ]
    .to_vec();

    let examples: Vec<_> = std::fs::read_dir("./rtic/examples")?
        .filter_map(|p| p.ok())
        .map(|p| p.path())
        .filter(|p| p.display().to_string().ends_with(".rs"))
        .map(|path| path.file_stem().unwrap().to_str().unwrap().to_string())
        .collect();

    let cli = Cli::parse();

    let env_logger_default_level = match cli.verbose {
        0 => Env::default().default_filter_or("error"),
        1 => Env::default().default_filter_or("info"),
        2 => Env::default().default_filter_or("debug"),
        _ => Env::default().default_filter_or("trace"),
    };
    env_logger::Builder::from_env(env_logger_default_level)
        .format_module_path(false)
        .format_timestamp(None)
        .init();

    trace!("default logging level: {0}", cli.verbose);
    trace!("examples: {examples:?}");

    let target = cli.target;
    let example = cli.example;

    if let Some(example) = example {
        if examples.contains(&example) {
            info!("Testing example: {example}");
            // If we managed to filter, set the examples to test to only this one
            examples = vec![example]
        } else {
            error!(
                "\nThe example you specified is not available. Available examples are:\
                    \n{examples:#?}\n\
             By default if example flag is emitted, all examples are tested.",
            );
            process::exit(1);
        }
    }
    if let Some(target) = target {
        if targets.contains(&target) {
            debug!("\nTesting target: {target}");
            // If we managed to filter, set the targets to test to only this one
            targets = vec![target]
        } else {
            error!(
                "\nThe target you specified is not available. Available targets are:\
                    \n{targets:#?}\n\
             By default all targets are tested.",
            );
            process::exit(1);
        }
    }

    init_build_dir()?;
    #[allow(clippy::if_same_then_else)]
    let cargoarg = if log_enabled!(Level::Trace) {
        Some("-vv")
    } else if log_enabled!(Level::Debug) {
        Some("-v")
    } else if log_enabled!(Level::Info) {
        None
    } else if log_enabled!(Level::Warn) || log_enabled!(Level::Error) {
        Some("--quiet")
    } else {
        // Off case
        Some("--quiet")
    };

    match cli.command {
        Some(Commands::Size(arguments)) => {
            debug!("Measuring on target(s): {targets:?}");
            for t in &targets {
                info!("Measuring for target: {t:?}");
                build_and_check_size(&cargoarg, t, &examples, &arguments.sizearguments)?;
            }
        }
        Some(Commands::Qemu {
            overwrite_expected: overwrite,
        }) => {
            debug!("Testing on target(s): {targets:?}");
            for t in &targets {
                info!("Testing for target: {t:?}");
                run_test(&cargoarg, t, &examples, overwrite)?;
            }
        }
        Some(Commands::Build) => {
            debug!("Building for target(s): {targets:?}");
            for t in &targets {
                info!("Building for target: {t:?}");
                build_all(&cargoarg, t)?;
            }
        }
        Some(Commands::Check) => {
            debug!("Checking on target(s): {targets:?}");
            for t in &targets {
                info!("Checking on target: {t:?}");
                check_all(&cargoarg, t)?;
            }
        }
        None => {
            todo!();
        }
    }

    Ok(())
}

fn build_all(cargoarg: &Option<&str>, target: &str) -> anyhow::Result<()> {
    arm_example(
        &CargoCommand::BuildAll {
            cargoarg,
            target,
            features: DEFAULT_FEATURES,
            mode: BuildMode::Release,
        },
        false,
    )?;
    Ok(())
}

fn check_all(cargoarg: &Option<&str>, target: &str) -> anyhow::Result<()> {
    arm_example(
        &CargoCommand::CheckAll {
            cargoarg,
            target,
            features: DEFAULT_FEATURES,
        },
        false,
    )?;
    Ok(())
}

fn run_test(
    cargoarg: &Option<&str>,
    target: &str,
    examples: &[String],
    overwrite: bool,
) -> anyhow::Result<()> {
    examples.into_par_iter().for_each(|example| {
        let cmd = CargoCommand::Build {
            cargoarg: &Some("--quiet"),
            example,
            target,
            features: DEFAULT_FEATURES,
            mode: BuildMode::Release,
        };
        arm_example(&cmd, false).unwrap();

        let cmd = CargoCommand::Run {
            cargoarg,
            example,
            target,
            features: DEFAULT_FEATURES,
            mode: BuildMode::Release,
        };

        arm_example(&cmd, overwrite).unwrap();
    });

    Ok(())
}

fn build_and_check_size(
    cargoarg: &Option<&str>,
    target: &str,
    examples: &[String],
    size_arguments: &Option<Sizearguments>,
) -> anyhow::Result<()> {
    examples.into_par_iter().for_each(|example| {
        // Make sure the requested example(s) are built
        let cmd = CargoCommand::Build {
            cargoarg: &Some("--quiet"),
            example,
            target,
            features: DEFAULT_FEATURES,
            mode: BuildMode::Release,
        };
        arm_example(&cmd, false).unwrap();

        let cmd = CargoCommand::Size {
            cargoarg,
            example,
            target,
            features: DEFAULT_FEATURES,
            mode: BuildMode::Release,
            arguments: size_arguments.clone(),
        };
        arm_example(&cmd, false).unwrap();
    });

    Ok(())
}

// run example binary `example`
fn arm_example(command: &CargoCommand, overwrite: bool) -> anyhow::Result<()> {
    match *command {
        CargoCommand::Run { example, .. } => {
            let run_file = format!("{example}.run");
            let expected_output_file = ["rtic", "ci", "expected", &run_file]
                .iter()
                .collect::<PathBuf>()
                .into_os_string()
                .into_string()
                .map_err(TestRunError::PathConversionError)?;

            // cargo run <..>
            let cargo_run_result = run_command(command)?;
            info!("{}", cargo_run_result.output);

            // Create a file for the expected output if it does not exist or mismatches
            if overwrite {
                let result = run_successful(&cargo_run_result, &expected_output_file);
                if let Err(e) = result {
                    // FileError means the file did not exist or was unreadable
                    error!("Error: {e}");
                    let mut file_handle = File::create(&expected_output_file).map_err(|_| {
                        TestRunError::FileError {
                            file: expected_output_file.clone(),
                        }
                    })?;
                    info!("Flag --overwrite-expected enabled");
                    info!("Creating/updating file: {expected_output_file}");
                    file_handle.write_all(cargo_run_result.output.as_bytes())?;
                };
            } else {
                run_successful(&cargo_run_result, &expected_output_file)?;
            }
            Ok(())
        }
        CargoCommand::Build { .. } => {
            // cargo run <..>
            let cargo_build_result = run_command(command)?;
            if !cargo_build_result.output.is_empty() {
                info!("{}", cargo_build_result.output);
            }

            Ok(())
        }
        CargoCommand::BuildAll { .. } => {
            // cargo build --examples
            let cargo_build_result = run_command(command)?;
            if !cargo_build_result.output.is_empty() {
                info!("{}", cargo_build_result.output);
            }

            Ok(())
        }
        CargoCommand::CheckAll { .. } => {
            // cargo check --examples
            let cargo_check_result = run_command(command)?;
            if !cargo_check_result.output.is_empty() {
                info!("{}", cargo_check_result.output);
            }

            Ok(())
        }
        CargoCommand::Size { .. } => {
            // cargo size
            let cargo_size_result = run_command(command)?;
            if !cargo_size_result.output.is_empty() {
                info!("{}", cargo_size_result.output);
            }
            Ok(())
        }
    }
}
