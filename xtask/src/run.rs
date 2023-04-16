use std::{
    fs::File,
    io::Read,
    path::PathBuf,
    process::{Command, Stdio},
};

use crate::{
    argument_parsing::{Backends, BuildOrCheck, ExtraArguments, Globals, PackageOpt, TestMetadata},
    cargo_command::{BuildMode, CargoCommand},
    command_parser, RunResult, TestRunError,
};
use log::{error, info, Level};

#[cfg(feature = "rayon")]
use rayon::prelude::*;

use iters::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputMode {
    PipedAndCollected,
    Inherited,
}

impl From<OutputMode> for Stdio {
    fn from(value: OutputMode) -> Self {
        match value {
            OutputMode::PipedAndCollected => Stdio::piped(),
            OutputMode::Inherited => Stdio::inherit(),
        }
    }
}

#[derive(Debug)]
pub enum FinalRunResult<'c> {
    Success(CargoCommand<'c>, RunResult),
    Failed(CargoCommand<'c>, RunResult),
    CommandError(CargoCommand<'c>, anyhow::Error),
}

fn run_and_convert<'a>(
    (global, command, overwrite): (&Globals, CargoCommand<'a>, bool),
) -> FinalRunResult<'a> {
    // Run the command
    let result = command_parser(global, &command, overwrite);

    let output = match result {
        // If running the command succeeded without looking at any of the results,
        // log the data and see if the actual execution was succesfull too.
        Ok(result) => {
            if result.exit_status.success() {
                FinalRunResult::Success(command, result)
            } else {
                FinalRunResult::Failed(command, result)
            }
        }
        // If it didn't and some IO error occured, just panic
        Err(e) => FinalRunResult::CommandError(command, e),
    };

    log::trace!("Final result: {output:?}");

    output
}

pub trait CoalescingRunner<'c> {
    /// Run all the commands in this iterator, and coalesce the results into
    /// one error (if any individual commands failed)
    fn run_and_coalesce(self) -> Vec<FinalRunResult<'c>>;
}

#[cfg(not(feature = "rayon"))]
mod iters {
    use super::*;

    pub fn examples_iter(examples: &[String]) -> impl Iterator<Item = &String> {
        examples.into_iter()
    }

    impl<'g, 'c, I> CoalescingRunner<'c> for I
    where
        I: Iterator<Item = (&'g Globals, CargoCommand<'c>, bool)>,
    {
        fn run_and_coalesce(self) -> Vec<FinalRunResult<'c>> {
            self.map(run_and_convert).collect()
        }
    }
}

#[cfg(feature = "rayon")]
mod iters {
    use super::*;

    pub fn examples_iter(examples: &[String]) -> impl ParallelIterator<Item = &String> {
        examples.into_par_iter()
    }

    impl<'g, 'c, I> CoalescingRunner<'c> for I
    where
        I: ParallelIterator<Item = (&'g Globals, CargoCommand<'c>, bool)>,
    {
        fn run_and_coalesce(self) -> Vec<FinalRunResult<'c>> {
            self.map(run_and_convert).collect()
        }
    }
}

/// Cargo command to either build or check
pub fn cargo<'c>(
    globals: &Globals,
    operation: BuildOrCheck,
    cargoarg: &'c Option<&'c str>,
    package: &'c PackageOpt,
    backend: Backends,
) -> Vec<FinalRunResult<'c>> {
    let runner = package
        .packages()
        .flat_map(|package| {
            let target = backend.to_target();
            let features = package.features(target, backend, globals.partial);

            #[cfg(feature = "rayon")]
            {
                features.into_par_iter().map(move |f| (package, target, f))
            }

            #[cfg(not(feature = "rayon"))]
            {
                features.into_iter().map(move |f| (package, target, f))
            }
        })
        .map(move |(package, target, features)| {
            let target = target.into();
            let command = match operation {
                BuildOrCheck::Check => CargoCommand::Check {
                    cargoarg,
                    package: Some(package.name()),
                    target,
                    features,
                    mode: BuildMode::Release,
                    dir: None,
                },
                BuildOrCheck::Build => CargoCommand::Build {
                    cargoarg,
                    package: Some(package.name()),
                    target,
                    features,
                    mode: BuildMode::Release,
                    dir: None,
                },
            };

            (globals, command, false)
        });

    runner.run_and_coalesce()
}

/// Cargo command to build a usage example.
///
/// The usage examples are in examples/
pub fn cargo_usage_example(
    globals: &Globals,
    operation: BuildOrCheck,
    usage_examples: Vec<String>,
) -> Vec<FinalRunResult<'_>> {
    examples_iter(&usage_examples)
        .map(|example| {
            let path = format!("examples/{example}");

            let command = match operation {
                BuildOrCheck::Check => CargoCommand::Check {
                    cargoarg: &None,
                    mode: BuildMode::Release,
                    dir: Some(path.into()),
                    package: None,
                    target: None,
                    features: None,
                },
                BuildOrCheck::Build => CargoCommand::Build {
                    cargoarg: &None,
                    package: None,
                    target: None,
                    features: None,
                    mode: BuildMode::Release,
                    dir: Some(path.into()),
                },
            };
            (globals, command, false)
        })
        .run_and_coalesce()
}

/// Cargo command to either build or check all examples
///
/// The examples are in rtic/examples
pub fn cargo_example<'c>(
    globals: &Globals,
    operation: BuildOrCheck,
    cargoarg: &'c Option<&'c str>,
    backend: Backends,
    examples: &'c [String],
) -> Vec<FinalRunResult<'c>> {
    let runner = examples_iter(examples).map(|example| {
        let features = Some(backend.to_target().and_features(backend.to_rtic_feature()));

        let command = match operation {
            BuildOrCheck::Check => CargoCommand::ExampleCheck {
                cargoarg,
                example,
                target: Some(backend.to_target()),
                features,
                mode: BuildMode::Release,
            },
            BuildOrCheck::Build => CargoCommand::ExampleBuild {
                cargoarg,
                example,
                target: Some(backend.to_target()),
                features,
                mode: BuildMode::Release,
                dir: Some(PathBuf::from("./rtic")),
            },
        };
        (globals, command, false)
    });
    runner.run_and_coalesce()
}

/// Run cargo clippy on selected package
pub fn cargo_clippy<'c>(
    globals: &Globals,
    cargoarg: &'c Option<&'c str>,
    package: &'c PackageOpt,
    backend: Backends,
) -> Vec<FinalRunResult<'c>> {
    let runner = package
        .packages()
        .flat_map(|package| {
            let target = backend.to_target();
            let features = package.features(target, backend, globals.partial);

            #[cfg(feature = "rayon")]
            {
                features.into_par_iter().map(move |f| (package, target, f))
            }

            #[cfg(not(feature = "rayon"))]
            {
                features.into_iter().map(move |f| (package, target, f))
            }
        })
        .map(move |(package, target, features)| {
            let command = CargoCommand::Clippy {
                cargoarg,
                package: Some(package.name()),
                target: target.into(),
                features,
            };

            (globals, command, false)
        });

    runner.run_and_coalesce()
}

/// Run cargo fmt on selected package
pub fn cargo_format<'c>(
    globals: &Globals,
    cargoarg: &'c Option<&'c str>,
    package: &'c PackageOpt,
    check_only: bool,
) -> Vec<FinalRunResult<'c>> {
    let runner = package.packages().map(|p| {
        (
            globals,
            CargoCommand::Format {
                cargoarg,
                package: Some(p.name()),
                check_only,
            },
            false,
        )
    });
    runner.run_and_coalesce()
}

/// Run cargo doc
pub fn cargo_doc<'c>(
    globals: &Globals,
    cargoarg: &'c Option<&'c str>,
    backend: Backends,
    arguments: &'c Option<ExtraArguments>,
) -> Vec<FinalRunResult<'c>> {
    let features = Some(backend.to_target().and_features(backend.to_rtic_feature()));

    let command = CargoCommand::Doc {
        cargoarg,
        features,
        arguments: arguments.clone(),
    };

    vec![run_and_convert((globals, command, false))]
}

/// Run cargo test on the selected package or all packages
///
/// If no package is specified, loop through all packages
pub fn cargo_test<'c>(
    globals: &Globals,
    package: &'c PackageOpt,
    backend: Backends,
) -> Vec<FinalRunResult<'c>> {
    package
        .packages()
        .map(|p| (globals, TestMetadata::match_package(p, backend), false))
        .run_and_coalesce()
}

/// Use mdbook to build the book
pub fn cargo_book<'c>(
    globals: &Globals,
    arguments: &'c Option<ExtraArguments>,
) -> Vec<FinalRunResult<'c>> {
    vec![run_and_convert((
        globals,
        CargoCommand::Book {
            arguments: arguments.clone(),
        },
        false,
    ))]
}

/// Run examples
///
/// Supports updating the expected output via the overwrite argument
pub fn qemu_run_examples<'c>(
    globals: &Globals,
    cargoarg: &'c Option<&'c str>,
    backend: Backends,
    examples: &'c [String],
    overwrite: bool,
) -> Vec<FinalRunResult<'c>> {
    let target = backend.to_target();
    let features = Some(target.and_features(backend.to_rtic_feature()));

    examples_iter(examples)
        .flat_map(|example| {
            let target = target.into();
            let cmd_build = CargoCommand::ExampleBuild {
                cargoarg: &None,
                example,
                target,
                features: features.clone(),
                mode: BuildMode::Release,
                dir: Some(PathBuf::from("./rtic")),
            };

            let cmd_qemu = CargoCommand::Qemu {
                cargoarg,
                example,
                target,
                features: features.clone(),
                mode: BuildMode::Release,
                dir: Some(PathBuf::from("./rtic")),
            };

            #[cfg(not(feature = "rayon"))]
            {
                [cmd_build, cmd_qemu].into_iter()
            }

            #[cfg(feature = "rayon")]
            {
                [cmd_build, cmd_qemu].into_par_iter()
            }
        })
        .map(|cmd| (globals, cmd, overwrite))
        .run_and_coalesce()
}

/// Check the binary sizes of examples
pub fn build_and_check_size<'c>(
    globals: &Globals,
    cargoarg: &'c Option<&'c str>,
    backend: Backends,
    examples: &'c [String],
    arguments: &'c Option<ExtraArguments>,
) -> Vec<FinalRunResult<'c>> {
    let target = backend.to_target();
    let features = Some(target.and_features(backend.to_rtic_feature()));

    let runner = examples_iter(examples).map(|example| {
        let target = target.into();

        // Make sure the requested example(s) are built
        let cmd = CargoCommand::ExampleBuild {
            cargoarg: &Some("--quiet"),
            example,
            target,
            features: features.clone(),
            mode: BuildMode::Release,
            dir: Some(PathBuf::from("./rtic")),
        };
        if let Err(err) = command_parser(globals, &cmd, false) {
            error!("{err}");
        }

        let cmd = CargoCommand::ExampleSize {
            cargoarg,
            example,
            target,
            features: features.clone(),
            mode: BuildMode::Release,
            arguments: arguments.clone(),
            dir: Some(PathBuf::from("./rtic")),
        };
        (globals, cmd, false)
    });

    runner.run_and_coalesce()
}

pub fn run_command(command: &CargoCommand, stderr_mode: OutputMode) -> anyhow::Result<RunResult> {
    log::info!("👟 {command}");

    let mut process = Command::new(command.executable());

    process
        .args(command.args())
        .stdout(Stdio::piped())
        .stderr(stderr_mode);

    if let Some(dir) = command.chdir() {
        process.current_dir(dir.canonicalize()?);
    }

    let result = process.output()?;

    let exit_status = result.status;
    let stderr = String::from_utf8(result.stderr).unwrap_or("Not displayable".into());
    let stdout = String::from_utf8(result.stdout).unwrap_or("Not displayable".into());

    if command.print_stdout_intermediate() && exit_status.success() {
        log::info!("\n{}", stdout);
    }

    if exit_status.success() {
        log::info!("✅ Success.")
    } else {
        log::error!("❌ Command failed. Run to completion for the summary.");
    }

    Ok(RunResult {
        exit_status,
        stdout,
        stderr,
    })
}

/// Check if `run` was successful.
/// returns Ok in case the run went as expected,
/// Err otherwise
pub fn run_successful(run: &RunResult, expected_output_file: &str) -> Result<(), TestRunError> {
    let mut file_handle =
        File::open(expected_output_file).map_err(|_| TestRunError::FileError {
            file: expected_output_file.to_owned(),
        })?;
    let mut expected_output = String::new();
    file_handle
        .read_to_string(&mut expected_output)
        .map_err(|_| TestRunError::FileError {
            file: expected_output_file.to_owned(),
        })?;

    if expected_output != run.stdout {
        Err(TestRunError::FileCmpError {
            expected: expected_output.clone(),
            got: run.stdout.clone(),
        })
    } else if !run.exit_status.success() {
        Err(TestRunError::CommandError(run.clone()))
    } else {
        Ok(())
    }
}

pub fn handle_results(globals: &Globals, results: Vec<FinalRunResult>) -> Result<(), ()> {
    let errors = results.iter().filter_map(|r| {
        if let FinalRunResult::Failed(c, r) = r {
            Some((c, &r.stdout, &r.stderr))
        } else {
            None
        }
    });

    let successes = results.iter().filter_map(|r| {
        if let FinalRunResult::Success(c, r) = r {
            Some((c, &r.stdout, &r.stderr))
        } else {
            None
        }
    });

    let command_errors = results.iter().filter_map(|r| {
        if let FinalRunResult::CommandError(c, e) = r {
            Some((c, e))
        } else {
            None
        }
    });

    let log_stdout_stderr = |level: Level| {
        move |(cmd, stdout, stderr): (&CargoCommand, &String, &String)| {
            let cmd = cmd.as_cmd_string();
            if !stdout.is_empty() && !stderr.is_empty() {
                log::log!(level, "\n{cmd}\nStdout:\n{stdout}\nStderr:\n{stderr}");
            } else if !stdout.is_empty() {
                log::log!(level, "\n{cmd}\nStdout:\n{}", stdout.trim_end());
            } else if !stderr.is_empty() {
                log::log!(level, "\n{cmd}\nStderr:\n{}", stderr.trim_end());
            }
        }
    };

    successes.for_each(|(cmd, stdout, stderr)| {
        if globals.verbose > 0 {
            info!("✅ Success: {cmd}\n    {}", cmd.as_cmd_string());
        } else {
            info!("✅ Success: {cmd}");
        }

        log_stdout_stderr(Level::Debug)((cmd, stdout, stderr));
    });

    errors.clone().for_each(|(cmd, stdout, stderr)| {
        error!("❌ Failed: {cmd}\n    {}", cmd.as_cmd_string());
        log_stdout_stderr(Level::Error)((cmd, stdout, stderr));
    });

    command_errors
        .clone()
        .for_each(|(cmd, error)| error!("❌ Failed: {cmd}\n    {}\n{error}", cmd.as_cmd_string()));

    let ecount = errors.count() + command_errors.count();
    if ecount != 0 {
        log::error!("{ecount} commands failed.");
        Err(())
    } else {
        info!("🚀🚀🚀 All tasks succeeded 🚀🚀🚀");
        Ok(())
    }
}
