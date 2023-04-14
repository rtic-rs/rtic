mod argument_parsing;
mod build;
mod cargo_commands;
mod command;

use argument_parsing::{ExtraArguments, Globals, Package};
use clap::Parser;
use command::OutputMode;
use core::fmt;
use diffy::{create_patch, PatchFormatter};
use std::{
    error::Error,
    ffi::OsString,
    fs::File,
    io::prelude::*,
    path::{Path, PathBuf},
    process::ExitStatus,
    str,
};

use log::{error, info, log_enabled, trace, Level};

use crate::{
    argument_parsing::{Backends, BuildOrCheck, Cli, Commands},
    build::init_build_dir,
    cargo_commands::*,
    command::{handle_results, run_command, run_successful, CargoCommand},
};

#[derive(Debug, Clone, Copy)]
pub struct Target<'a> {
    triple: &'a str,
    has_std: bool,
}

impl<'a> Target<'a> {
    const DEFAULT_FEATURES: &str = "test-critical-section";

    pub const fn new(triple: &'a str, has_std: bool) -> Self {
        Self { triple, has_std }
    }

    pub fn triple(&self) -> &str {
        self.triple
    }

    pub fn has_std(&self) -> bool {
        self.has_std
    }

    pub fn and_features(&self, features: &str) -> String {
        format!("{},{}", Self::DEFAULT_FEATURES, features)
    }
}

impl core::fmt::Display for Target<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.triple)
    }
}

// x86_64-unknown-linux-gnu
const _X86_64: Target = Target::new("x86_64-unknown-linux-gnu", true);
const ARMV6M: Target = Target::new("thumbv6m-none-eabi", false);
const ARMV7M: Target = Target::new("thumbv7m-none-eabi", false);
const ARMV8MBASE: Target = Target::new("thumbv8m.base-none-eabi", false);
const ARMV8MMAIN: Target = Target::new("thumbv8m.main-none-eabi", false);

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
        return Err(anyhow::anyhow!(
            "xtasks can only be executed from the root of the `rtic` repository"
        ));
    }

    let examples: Vec<_> = std::fs::read_dir("./rtic/examples")?
        .filter_map(|p| p.ok())
        .map(|p| p.path())
        .filter(|p| p.display().to_string().ends_with(".rs"))
        .map(|path| path.file_stem().unwrap().to_str().unwrap().to_string())
        .collect();

    let cli = Cli::parse();

    let globals = &cli.globals;

    let env_logger_default_level = match globals.verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };

    pretty_env_logger::formatted_builder()
        .parse_filters(&std::env::var("RUST_LOG").unwrap_or(env_logger_default_level.into()))
        .init();

    trace!("default logging level: {0}", globals.verbose);

    log::debug!(
        "Stderr of child processes is inherited: {}",
        globals.stderr_inherited
    );
    log::debug!("Partial features: {}", globals.partial);

    let backend = if let Some(backend) = globals.backend {
        backend
    } else {
        Backends::default()
    };

    let example = globals.example.clone();
    let exampleexclude = globals.exampleexclude.clone();

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
            return Err(anyhow::anyhow!("Incorrect usage"));
        } else {
            examples_to_run
        }
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

    let final_run_results = match &cli.command {
        Commands::Format(args) => cargo_format(globals, &cargologlevel, &args.package, args.check),
        Commands::Clippy(args) => {
            info!("Running clippy on backend: {backend:?}");
            cargo_clippy(globals, &cargologlevel, &args, backend)
        }
        Commands::Check(args) => {
            info!("Checking on backend: {backend:?}");
            cargo(globals, BuildOrCheck::Check, &cargologlevel, &args, backend)
        }
        Commands::Build(args) => {
            info!("Building for backend: {backend:?}");
            cargo(globals, BuildOrCheck::Build, &cargologlevel, &args, backend)
        }
        Commands::ExampleCheck => {
            info!("Checking on backend: {backend:?}");
            cargo_example(
                globals,
                BuildOrCheck::Check,
                &cargologlevel,
                backend,
                &examples_to_run,
            )
        }
        Commands::ExampleBuild => {
            info!("Building for backend: {backend:?}");
            cargo_example(
                globals,
                BuildOrCheck::Build,
                &cargologlevel,
                backend,
                &examples_to_run,
            )
        }
        Commands::Size(args) => {
            // x86_64 target not valid
            info!("Measuring for backend: {backend:?}");
            build_and_check_size(
                globals,
                &cargologlevel,
                backend,
                &examples_to_run,
                &args.arguments,
            )
        }
        Commands::Qemu(args) | Commands::Run(args) => {
            // x86_64 target not valid
            info!("Testing for backend: {backend:?}");
            run_test(
                globals,
                &cargologlevel,
                backend,
                &examples_to_run,
                args.overwrite_expected,
            )
        }
        Commands::Doc(args) => {
            info!("Running cargo doc on backend: {backend:?}");
            cargo_doc(globals, &cargologlevel, backend, &args.arguments)
        }
        Commands::Test(args) => {
            info!("Running cargo test on backend: {backend:?}");
            cargo_test(globals, &args, backend)
        }
        Commands::Book(args) => {
            info!("Running mdbook");
            cargo_book(globals, &args.arguments)
        }
        Commands::UsageExamplesCheck(examples) => {
            info!("Checking usage examples");
            cargo_usage_example(globals, BuildOrCheck::Check, examples.examples()?)
        }
        Commands::UsageExampleBuild(examples) => {
            info!("Building usage examples");
            cargo_usage_example(globals, BuildOrCheck::Build, examples.examples()?)
        }
    };

    handle_results(globals, final_run_results)
}

// run example binary `example`
fn command_parser(
    glob: &Globals,
    command: &CargoCommand,
    overwrite: bool,
) -> anyhow::Result<RunResult> {
    let output_mode = if glob.stderr_inherited {
        OutputMode::Inherited
    } else {
        OutputMode::PipedAndCollected
    };

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
            let cargo_run_result = run_command(command, output_mode)?;
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
            };

            Ok(cargo_run_result)
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
        | CargoCommand::ExampleSize { .. }
        | CargoCommand::BuildInDir { .. }
        | CargoCommand::CheckInDir { .. } => {
            let cargo_result = run_command(command, output_mode)?;
            Ok(cargo_result)
        }
    }
}
