mod argument_parsing;
mod build;
mod cargo_commands;
mod command;

use anyhow::bail;
use argument_parsing::{ExtraArguments, Package};
use clap::Parser;
use core::fmt;
use diffy::{create_patch, PatchFormatter};
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
    argument_parsing::{Backends, BuildOrCheck, Cli, Commands, PackageOpt},
    build::init_build_dir,
    cargo_commands::{
        build_and_check_size, cargo, cargo_book, cargo_clippy, cargo_doc, cargo_example,
        cargo_format, cargo_test, run_test,
    },
    command::{run_command, run_successful, CargoCommand},
};

// x86_64-unknown-linux-gnu
const _X86_64: &str = "x86_64-unknown-linux-gnu";
const ARMV6M: &str = "thumbv6m-none-eabi";
const ARMV7M: &str = "thumbv7m-none-eabi";
const ARMV8MBASE: &str = "thumbv8m.base-none-eabi";
const ARMV8MMAIN: &str = "thumbv8m.main-none-eabi";

const DEFAULT_FEATURES: &str = "test-critical-section";

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

    let backend = if let Some(backend) = cli.backend {
        backend
    } else {
        Backends::default()
    };

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
    let cargologlevel = if log_enabled!(Level::Trace) {
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
        Commands::FormatCheck(args) => {
            info!("Running cargo fmt: {args:?}");
            let check_only = true;
            cargo_format(&cargologlevel, &args, check_only)?;
        }
        Commands::Format(args) => {
            info!("Running cargo fmt --check: {args:?}");
            let check_only = false;
            cargo_format(&cargologlevel, &args, check_only)?;
        }
        Commands::Clippy(args) => {
            info!("Running clippy on backend: {backend:?}");
            cargo_clippy(&cargologlevel, &args, backend)?;
        }
        Commands::Check(args) => {
            info!("Checking on backend: {backend:?}");
            cargo(BuildOrCheck::Check, &cargologlevel, &args, backend)?;
        }
        Commands::Build(args) => {
            info!("Building for backend: {backend:?}");
            cargo(BuildOrCheck::Build, &cargologlevel, &args, backend)?;
        }
        Commands::ExampleCheck => {
            info!("Checking on backend: {backend:?}");
            cargo_example(
                BuildOrCheck::Check,
                &cargologlevel,
                backend,
                &examples_to_run,
            )?;
        }
        Commands::ExampleBuild => {
            info!("Building for backend: {backend:?}");
            cargo_example(
                BuildOrCheck::Build,
                &cargologlevel,
                backend,
                &examples_to_run,
            )?;
        }
        Commands::Size(args) => {
            // x86_64 target not valid
            info!("Measuring for backend: {backend:?}");
            build_and_check_size(&cargologlevel, backend, &examples_to_run, &args.arguments)?;
        }
        Commands::Qemu(args) | Commands::Run(args) => {
            // x86_64 target not valid
            info!("Testing for backend: {backend:?}");
            run_test(
                &cargologlevel,
                backend,
                &examples_to_run,
                args.overwrite_expected,
            )?;
        }
        Commands::Doc(args) => {
            info!("Running cargo doc on backend: {backend:?}");
            cargo_doc(&cargologlevel, backend, &args.arguments)?;
        }
        Commands::Test(args) => {
            info!("Running cargo test on backend: {backend:?}");
            cargo_test(&args, backend)?;
        }
        Commands::Book(args) => {
            info!("Running mdbook");
            cargo_book(&args.arguments)?;
        }
    }

    Ok(())
}

/// Get the features needed given the selected package
///
/// Without package specified the features for RTIC are required
/// With only a single package which is not RTIC, no special
/// features are needed
fn package_feature_extractor(package: &PackageOpt, backend: Backends) -> Option<String> {
    let default_features = Some(format!(
        "{},{}",
        DEFAULT_FEATURES,
        backend.to_rtic_feature()
    ));
    if let Some(package) = package.package {
        debug!("\nTesting package: {package}");
        match package {
            Package::Rtic => default_features,
            Package::RticMacros => Some(backend.to_rtic_macros_feature().to_owned()),
            _ => None,
        }
    } else {
        default_features
    }
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
        CargoCommand::Format { .. }
        | CargoCommand::ExampleCheck { .. }
        | CargoCommand::ExampleBuild { .. }
        | CargoCommand::Check { .. }
        | CargoCommand::Build { .. }
        | CargoCommand::Clippy { .. }
        | CargoCommand::Doc { .. }
        | CargoCommand::Test { .. }
        | CargoCommand::Book { .. }
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
