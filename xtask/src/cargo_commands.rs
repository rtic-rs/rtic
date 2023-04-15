use crate::{
    argument_parsing::{Backends, BuildOrCheck, ExtraArguments, Globals, PackageOpt, TestMetadata},
    command::{BuildMode, CargoCommand},
    command_parser, RunResult,
};
use log::{error, info, Level};

#[cfg(feature = "rayon")]
use rayon::prelude::*;

use iters::*;

enum FinalRunResult<'c> {
    Success(CargoCommand<'c>, RunResult),
    Failed(CargoCommand<'c>, RunResult),
    CommandError(anyhow::Error),
}

fn run_and_convert<'a>(
    (global, command, overwrite): (&Globals, CargoCommand<'a>, bool),
) -> FinalRunResult<'a> {
    // Run the command
    let result = command_parser(global, &command, overwrite);
    match result {
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
        Err(e) => FinalRunResult::CommandError(e),
    }
}

fn handle_results(results: Vec<FinalRunResult>) -> anyhow::Result<()> {
    let errors = results.iter().filter_map(|r| {
        if let FinalRunResult::Failed(c, r) = r {
            Some((c, r))
        } else {
            None
        }
    });

    let successes = results.iter().filter_map(|r| {
        if let FinalRunResult::Success(c, r) = r {
            Some((c, r))
        } else {
            None
        }
    });

    let log_stdout_stderr = |level: Level| {
        move |(command, result): (&CargoCommand, &RunResult)| {
            let stdout = &result.stdout;
            let stderr = &result.stderr;
            if !stdout.is_empty() && !stderr.is_empty() {
                log::log!(
                    level,
                    "Command output for {command}\nStdout:\n{stdout}\nStderr:\n{stderr}"
                );
            } else if !stdout.is_empty() {
                log::log!(
                    level,
                    "Command output for {command}\nStdout:\n{}",
                    stdout.trim_end()
                );
            } else if !stderr.is_empty() {
                log::log!(
                    level,
                    "Command output for {command}\nStderr:\n{}",
                    stderr.trim_end()
                );
            }
        }
    };

    successes.clone().for_each(log_stdout_stderr(Level::Debug));
    errors.clone().for_each(log_stdout_stderr(Level::Error));

    successes.for_each(|(cmd, _)| {
        info!("Succesfully executed {cmd}");
    });

    errors.clone().for_each(|(cmd, _)| {
        error!("Command {cmd} failed");
    });

    if errors.count() != 0 {
        Err(anyhow::anyhow!("Some commands failed."))
    } else {
        Ok(())
    }
}

pub trait CoalescingRunning {
    /// Run all the commands in this iterator, and coalesce the results into
    /// one error (if any individual commands failed)
    fn run_and_coalesce(self) -> anyhow::Result<()>;
}

#[cfg(not(feature = "rayon"))]
mod iters {
    use super::*;

    pub fn examples_iter(examples: &[String]) -> impl Iterator<Item = &String> {
        examples.into_iter()
    }

    impl<'g, 'c, I> CoalescingRunning for I
    where
        I: Iterator<Item = (&'g Globals, CargoCommand<'c>, bool)>,
    {
        fn run_and_coalesce(self) -> anyhow::Result<()> {
            let results: Vec<_> = self.map(run_and_convert).collect();
            handle_results(results)
        }
    }
}

#[cfg(feature = "rayon")]
mod iters {
    use super::*;

    pub fn examples_iter(examples: &[String]) -> impl ParallelIterator<Item = &String> {
        examples.into_par_iter()
    }

    impl<'g, 'c, I> CoalescingRunning for I
    where
        I: ParallelIterator<Item = (&'g Globals, CargoCommand<'c>, bool)>,
    {
        fn run_and_coalesce(self) -> anyhow::Result<()> {
            let results: Vec<_> = self.map(run_and_convert).collect();
            handle_results(results)
        }
    }
}

/// Cargo command to either build or check
pub fn cargo(
    globals: &Globals,
    operation: BuildOrCheck,
    cargoarg: &Option<&str>,
    package: &PackageOpt,
    backend: Backends,
) -> anyhow::Result<()> {
    let runner = package.packages().map(|package| {
        let target = backend.to_target();

        let features = package.extract_features(target, backend);

        match operation {
            BuildOrCheck::Check => {
                log::debug!(target: "xtask::command", "Checking package: {package}")
            }
            BuildOrCheck::Build => {
                log::debug!(target: "xtask::command", "Building package: {package}")
            }
        }

        let command = match operation {
            BuildOrCheck::Check => CargoCommand::Check {
                cargoarg,
                package: Some(package),
                target,
                features,
                mode: BuildMode::Release,
            },
            BuildOrCheck::Build => CargoCommand::Build {
                cargoarg,
                package: Some(package),
                target,
                features,
                mode: BuildMode::Release,
            },
        };

        (globals, command, false)
    });

    runner.run_and_coalesce()
}

/// Cargo command to either build or check all examples
///
/// The examples are in rtic/examples
pub fn cargo_example(
    globals: &Globals,
    operation: BuildOrCheck,
    cargoarg: &Option<&str>,
    backend: Backends,
    examples: &[String],
) -> anyhow::Result<()> {
    let runner = examples_iter(examples).map(|example| {
        let features = Some(backend.to_target().and_features(backend.to_rtic_feature()));

        let command = match operation {
            BuildOrCheck::Check => CargoCommand::ExampleCheck {
                cargoarg,
                example,
                target: backend.to_target(),
                features,
                mode: BuildMode::Release,
            },
            BuildOrCheck::Build => CargoCommand::ExampleBuild {
                cargoarg,
                example,
                target: backend.to_target(),
                features,
                mode: BuildMode::Release,
            },
        };
        (globals, command, false)
    });
    runner.run_and_coalesce()
}

/// Run cargo clippy on selected package
pub fn cargo_clippy(
    globals: &Globals,
    cargoarg: &Option<&str>,
    package: &PackageOpt,
    backend: Backends,
) -> anyhow::Result<()> {
    let runner = package.packages().map(|p| {
        let target = backend.to_target();
        let features = p.extract_features(target, backend);

        (
            globals,
            CargoCommand::Clippy {
                cargoarg,
                package: Some(p),
                target,
                features,
            },
            false,
        )
    });

    runner.run_and_coalesce()
}

/// Run cargo fmt on selected package
pub fn cargo_format(
    globals: &Globals,
    cargoarg: &Option<&str>,
    package: &PackageOpt,
    check_only: bool,
) -> anyhow::Result<()> {
    let runner = package.packages().map(|p| {
        (
            globals,
            CargoCommand::Format {
                cargoarg,
                package: Some(p),
                check_only,
            },
            false,
        )
    });
    runner.run_and_coalesce()
}

/// Run cargo doc
pub fn cargo_doc(
    globals: &Globals,
    cargoarg: &Option<&str>,
    backend: Backends,
    arguments: &Option<ExtraArguments>,
) -> anyhow::Result<()> {
    let features = Some(backend.to_target().and_features(backend.to_rtic_feature()));

    command_parser(
        globals,
        &CargoCommand::Doc {
            cargoarg,
            features,
            arguments: arguments.clone(),
        },
        false,
    )?;
    Ok(())
}

/// Run cargo test on the selected package or all packages
///
/// If no package is specified, loop through all packages
pub fn cargo_test(
    globals: &Globals,
    package: &PackageOpt,
    backend: Backends,
) -> anyhow::Result<()> {
    package
        .packages()
        .map(|p| (globals, TestMetadata::match_package(p, backend), false))
        .run_and_coalesce()
}

/// Use mdbook to build the book
pub fn cargo_book(
    globals: &Globals,
    arguments: &Option<ExtraArguments>,
) -> anyhow::Result<RunResult> {
    command_parser(
        globals,
        &CargoCommand::Book {
            arguments: arguments.clone(),
        },
        false,
    )
}

/// Run examples
///
/// Supports updating the expected output via the overwrite argument
pub fn run_test(
    globals: &Globals,
    cargoarg: &Option<&str>,
    backend: Backends,
    examples: &[String],
    overwrite: bool,
) -> anyhow::Result<()> {
    let target = backend.to_target();
    let features = Some(target.and_features(backend.to_rtic_feature()));

    examples_iter(examples)
        .map(|example| {
            let cmd = CargoCommand::ExampleBuild {
                cargoarg: &Some("--quiet"),
                example,
                target,
                features: features.clone(),
                mode: BuildMode::Release,
            };

            if let Err(err) = command_parser(globals, &cmd, false) {
                error!("{err}");
            }

            let cmd = CargoCommand::Qemu {
                cargoarg,
                example,
                target,
                features: features.clone(),
                mode: BuildMode::Release,
            };

            (globals, cmd, overwrite)
        })
        .run_and_coalesce()
}

/// Check the binary sizes of examples
pub fn build_and_check_size(
    globals: &Globals,
    cargoarg: &Option<&str>,
    backend: Backends,
    examples: &[String],
    arguments: &Option<ExtraArguments>,
) -> anyhow::Result<()> {
    let target = backend.to_target();
    let features = Some(target.and_features(backend.to_rtic_feature()));

    let runner = examples_iter(examples).map(|example| {
        // Make sure the requested example(s) are built
        let cmd = CargoCommand::ExampleBuild {
            cargoarg: &Some("--quiet"),
            example,
            target,
            features: features.clone(),
            mode: BuildMode::Release,
        };
        if let Err(err) = command_parser(globals, &cmd, false) {
            error!("{err}");
        }

        let cmd = CargoCommand::ExampleSize {
            cargoarg,
            example,
            target: backend.to_target(),
            features: features.clone(),
            mode: BuildMode::Release,
            arguments: arguments.clone(),
        };
        (globals, cmd, false)
    });

    runner.run_and_coalesce()
}
