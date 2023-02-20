mod build;
mod command;

use anyhow::bail;
use clap::{Args, Parser, Subcommand};
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
use exitcode;
use log::{debug, error, info, log_enabled, trace, Level};

use crate::{
    build::init_build_dir,
    command::{run_command, run_successful, BuildMode, CargoCommand},
};

// x86_64-unknown-linux-gnu
const _X86_64: &str = "x86_64-unknown-linux-gnu";
const ARMV6M: &str = "thumbv6m-none-eabi";
const ARMV7M: &str = "thumbv7m-none-eabi";
const ARMV8MBASE: &str = "thumbv8m.base-none-eabi";
const ARMV8MMAIN: &str = "thumbv8m.main-none-eabi";

const DEFAULT_FEATURES: &str = "test-critical-section";

#[derive(clap::ValueEnum, Copy, Clone, Default, Debug)]
enum Backends {
    Thumbv6,
    #[default]
    Thumbv7,
    Thumbv8Base,
    Thumbv8Main,
}

impl Backends {
    fn to_target(&self) -> &str {
        match self {
            Backends::Thumbv6 => ARMV6M,
            Backends::Thumbv7 => ARMV7M,
            Backends::Thumbv8Base => ARMV8MBASE,
            Backends::Thumbv8Main => ARMV8MMAIN,
        }
    }

    fn to_rtic_feature(&self) -> &str {
        match self {
            Backends::Thumbv6 => "thumbv6-backend",
            Backends::Thumbv7 => "thumbv7-backend",
            Backends::Thumbv8Base => "thumbv8base-backend",
            Backends::Thumbv8Main => "thumbv8main-backend",
        }
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
/// RTIC xtask powered testing toolbox
struct Cli {
    /// For which backend to build
    #[arg(value_enum, short, long)]
    backend: Backends,

    /// List of comma separated examples to include, all others are excluded
    ///
    /// If omitted all examples are included
    ///
    /// Example: `cargo xtask --example complex,spawn,init`
    /// would include complex, spawn and init
    #[arg(short, long, group = "example_group")]
    example: Option<String>,

    /// List of comma separated examples to exclude, all others are included
    ///
    /// If omitted all examples are included
    ///
    /// Example: `cargo xtask --excludeexample complex,spawn,init`
    /// would exclude complex, spawn and init
    #[arg(long, group = "example_group")]
    exampleexclude: Option<String>,

    /// Enable more verbose output, repeat up to `-vvv` for even more
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Subcommand selecting operation
    #[command(subcommand)]
    command: Commands,
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
    ///
    /// Requires that an ARM target is selected
    Qemu(QemuAndRun),

    /// Run examples through embedded-ci and compare against expected output
    ///
    /// unimplemented!() For now TODO, equal to Qemu
    ///
    /// Example runtime output is matched against `rtic/ci/expected/`
    ///
    /// Requires that an ARM target is selected
    Run(QemuAndRun),

    /// Build all examples
    ExampleBuild,

    /// Check all packages
    ExampleCheck,

    /// Build all examples
    Build(Package),

    /// Check all packages
    Check(Package),

    /// Run clippy
    Clippy(Package),
}

#[derive(Args, Debug)]
/// Restrict to package, or run on whole workspace
struct Package {
    /// For which package/workspace member to operate
    ///
    /// If omitted, work on all
    package: Option<String>,
}

#[derive(Args, Debug)]
struct QemuAndRun {
    /// If expected output is missing or mismatching, recreate the file
    ///
    /// This overwrites only missing or mismatching
    #[arg(long)]
    overwrite_expected: bool,
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
    stdout: String,
    stderr: String,
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
                    e.exit_status, e.stdout
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

    let backend = cli.backend;

    let example = cli.example;
    let exampleexclude = cli.exampleexclude;

    let examples_to_run = {
        let mut examples_to_run = examples.clone();

        if let Some(example) = example {
            examples_to_run = examples.clone();
            let examples_to_exclude = example.split(',').collect::<Vec<&str>>();
            // From the list of all examples, remove all not listed as included
            for ex in examples_to_exclude {
                examples_to_run.retain(|x| *x.as_str() == *ex);
            }
        };

        if let Some(example) = exampleexclude {
            examples_to_run = examples.clone();
            let examples_to_exclude = example.split(',').collect::<Vec<&str>>();
            // From the list of all examples, remove all those listed as excluded
            for ex in examples_to_exclude {
                examples_to_run.retain(|x| *x.as_str() != *ex);
            }
        };

        if log_enabled!(Level::Trace) {
            trace!("All examples:\n{examples:?} number: {}", examples.len());
            trace!(
                "examples_to_run:\n{examples_to_run:?} number: {}",
                examples_to_run.len()
            );
        }

        if examples_to_run.is_empty() {
            error!(
                "\nThe example(s) you specified is not available. Available examples are:\
                    \n{examples:#?}\n\
             By default if example flag is emitted, all examples are tested.",
            );
            process::exit(exitcode::USAGE);
        } else {
        }
        examples_to_run
    };

    init_build_dir()?;
    #[allow(clippy::if_same_then_else)]
    let cargoarg = if log_enabled!(Level::Trace) {
        Some("-v")
    } else if log_enabled!(Level::Debug) {
        None
    } else if log_enabled!(Level::Info) {
        None
    } else if log_enabled!(Level::Warn) || log_enabled!(Level::Error) {
        None
    } else {
        // Off case
        Some("--quiet")
    };

    match cli.command {
        Commands::Size(arguments) => {
            // x86_64 target not valid
            info!("Measuring for backend: {backend:?}");
            build_and_check_size(
                &cargoarg,
                backend,
                &examples_to_run,
                &arguments.sizearguments,
            )?;
        }
        Commands::Qemu(args) | Commands::Run(args) => {
            // x86_64 target not valid
            info!("Testing for backend: {backend:?}");
            run_test(
                &cargoarg,
                backend,
                &examples_to_run,
                args.overwrite_expected,
            )?;
        }
        Commands::ExampleBuild => {
            info!("Building for backend: {backend:?}");
            example_build(&cargoarg, backend, &examples_to_run)?;
        }
        Commands::ExampleCheck => {
            info!("Checking on backend: {backend:?}");
            example_check(&cargoarg, backend, &examples_to_run)?;
        }
        Commands::Build(args) => {
            info!("Building for backend: {backend:?}");
            cargo_build(&cargoarg, &args, backend)?;
        }
        Commands::Check(args) => {
            info!("Checking on backend: {backend:?}");
            cargo_check(&cargoarg, &args, backend)?;
        }
        Commands::Clippy(args) => {
            info!("Running clippy on backend: {backend:?}");
            cargo_clippy(&cargoarg, &args, backend)?;
        }
    }

    Ok(())
}

fn cargo_build(
    cargoarg: &Option<&str>,
    package: &Package,
    backend: Backends,
) -> anyhow::Result<()> {
    command_parser(
        &CargoCommand::Build {
            cargoarg,
            package: package_filter(package),
            target: backend.to_target(),
            features: None,
            mode: BuildMode::Release,
        },
        false,
    )?;
    Ok(())
}

fn cargo_check(
    cargoarg: &Option<&str>,
    package: &Package,
    backend: Backends,
) -> anyhow::Result<()> {
    command_parser(
        &CargoCommand::Check {
            cargoarg,
            package: package_filter(package),
            target: backend.to_target(),
            features: None,
        },
        false,
    )?;
    Ok(())
}

fn cargo_clippy(
    cargoarg: &Option<&str>,
    package: &Package,
    backend: Backends,
) -> anyhow::Result<()> {
    command_parser(
        &CargoCommand::Clippy {
            cargoarg,
            package: package_filter(package),
            target: backend.to_target(),
            features: None,
        },
        false,
    )?;
    Ok(())
}

fn run_test(
    cargoarg: &Option<&str>,
    backend: Backends,
    examples: &[String],
    overwrite: bool,
) -> anyhow::Result<()> {
    let s = format!("{},{}", DEFAULT_FEATURES, backend.to_rtic_feature());
    let features: Option<&str> = Some(&s);

    examples.into_par_iter().for_each(|example| {
        let cmd = CargoCommand::ExampleBuild {
            cargoarg: &Some("--quiet"),
            example,
            target: backend.to_target(),
            features,
            mode: BuildMode::Release,
        };
        if let Err(err) = command_parser(&cmd, false) {
            error!("{err}");
        }

        let cmd = CargoCommand::Qemu {
            cargoarg,
            example,
            target: backend.to_target(),
            features,
            mode: BuildMode::Release,
        };

        if let Err(err) = command_parser(&cmd, overwrite) {
            error!("{err}");
        }
    });

    Ok(())
}
fn example_check(
    cargoarg: &Option<&str>,
    backend: Backends,
    examples: &[String],
) -> anyhow::Result<()> {
    let s = format!("{},{}", DEFAULT_FEATURES, backend.to_rtic_feature());
    let features: Option<&str> = Some(&s);

    examples.into_par_iter().for_each(|example| {
        let cmd = CargoCommand::ExampleCheck {
            cargoarg,
            example,
            target: backend.to_target(),
            features,
            mode: BuildMode::Release,
        };

        if let Err(err) = command_parser(&cmd, false) {
            error!("{err}");
        }
    });

    Ok(())
}

fn example_build(
    cargoarg: &Option<&str>,
    backend: Backends,
    examples: &[String],
) -> anyhow::Result<()> {
    let s = format!("{},{}", DEFAULT_FEATURES, backend.to_rtic_feature());
    let features: Option<&str> = Some(&s);

    examples.into_par_iter().for_each(|example| {
        let cmd = CargoCommand::ExampleBuild {
            cargoarg,
            example,
            target: backend.to_target(),
            features,
            mode: BuildMode::Release,
        };

        if let Err(err) = command_parser(&cmd, false) {
            error!("{err}");
        }
    });

    Ok(())
}

fn build_and_check_size(
    cargoarg: &Option<&str>,
    backend: Backends,
    examples: &[String],
    size_arguments: &Option<Sizearguments>,
) -> anyhow::Result<()> {
    let s = format!("{},{}", DEFAULT_FEATURES, backend.to_rtic_feature());
    let features: Option<&str> = Some(&s);

    examples.into_par_iter().for_each(|example| {
        // Make sure the requested example(s) are built
        let cmd = CargoCommand::ExampleBuild {
            cargoarg: &Some("--quiet"),
            example,
            target: backend.to_target(),
            features,
            mode: BuildMode::Release,
        };
        if let Err(err) = command_parser(&cmd, false) {
            error!("{err}");
        }

        let cmd = CargoCommand::ExampleSize {
            cargoarg,
            example,
            target: backend.to_target(),
            features,
            mode: BuildMode::Release,
            arguments: size_arguments.clone(),
        };
        if let Err(err) = command_parser(&cmd, false) {
            error!("{err}");
        }
    });

    Ok(())
}

fn package_filter(package: &Package) -> Vec<String> {
    // TODO Parse Cargo.toml workspace definition instead?
    let packages: Vec<String> = [
        "rtic".to_owned(),
        "rtic-arbiter".to_owned(),
        "rtic-channel".to_owned(),
        "rtic-common".to_owned(),
        "rtic-macros".to_owned(),
        "rtic-monotonics".to_owned(),
        "rtic-time".to_owned(),
    ]
    .to_vec();

    let package_selected;

    if let Some(package) = package.package.clone() {
        if packages.contains(&package) {
            debug!("\nTesting package: {package}");
            // If we managed to filter, set the packages to test to only this one
            package_selected = vec![package]
        } else {
            error!(
                "\nThe package you specified is not available. Available packages are:\
                    \n{packages:#?}\n\
             By default all packages are tested.",
            );
            process::exit(exitcode::USAGE);
        }
    } else {
        package_selected = packages;
    }
    package_selected
}

// run example binary `example`
fn command_parser(command: &CargoCommand, overwrite: bool) -> anyhow::Result<()> {
    match *command {
        CargoCommand::Qemu { example, .. } | CargoCommand::Run { example, .. } => {
            let run_file = format!("{example}.run");
            let expected_output_file = ["rtic", "ci", "expected", &run_file]
                .iter()
                .collect::<PathBuf>()
                .into_os_string()
                .into_string()
                .map_err(TestRunError::PathConversionError)?;

            // cargo run <..>
            info!("Running example: {example}");
            let cargo_run_result = run_command(command)?;
            info!("{}", cargo_run_result.stdout);

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
                    file_handle.write_all(cargo_run_result.stdout.as_bytes())?;
                };
            } else {
                run_successful(&cargo_run_result, &expected_output_file)?;
            }
            Ok(())
        }
        CargoCommand::ExampleBuild { .. }
        | CargoCommand::ExampleCheck { .. }
        | CargoCommand::Build { .. }
        | CargoCommand::Check { .. }
        | CargoCommand::Clippy { .. }
        | CargoCommand::ExampleSize { .. } => {
            let cargo_result = run_command(command)?;
            if let Some(exit_code) = cargo_result.exit_status.code() {
                if exit_code != exitcode::OK {
                    error!("Exit code from command: {exit_code}");
                    if !cargo_result.stdout.is_empty() {
                        info!("{}", cargo_result.stdout);
                    }
                    if !cargo_result.stderr.is_empty() {
                        error!("{}", cargo_result.stderr);
                    }
                    process::exit(exit_code);
                } else {
                    if !cargo_result.stdout.is_empty() {
                        info!("{}", cargo_result.stdout);
                    }
                    if !cargo_result.stderr.is_empty() {
                        info!("{}", cargo_result.stderr);
                    }
                }
            }

            Ok(())
        }
    }
}
