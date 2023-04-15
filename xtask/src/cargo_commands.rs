use crate::{
    argument_parsing::{Backends, BuildOrCheck, ExtraArguments, Globals, PackageOpt, TestMetadata},
    command::{BuildMode, CargoCommand},
    command_parser, RunResult,
};
use log::error;

#[cfg(feature = "rayon")]
use rayon::prelude::*;

use iters::*;

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
                BuildOrCheck::Check => CargoCommand::CheckInDir {
                    mode: BuildMode::Release,
                    dir: path.into(),
                },
                BuildOrCheck::Build => CargoCommand::BuildInDir {
                    mode: BuildMode::Release,
                    dir: path.into(),
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
            (
                globals,
                CargoCommand::Clippy {
                    cargoarg,
                    package: Some(package),
                    target,
                    features,
                },
                false,
            )
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
                package: Some(p),
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

    let build = examples_iter(examples).map(|example| {
        let cmd_build = CargoCommand::ExampleBuild {
            // We need to be in the correct
            cargoarg: &None,
            example,
            target,
            features: features.clone(),
            mode: BuildMode::Release,
        };
        (globals, cmd_build, overwrite)
    });

    let run = examples_iter(examples).map(|example| {
        let cmd_qemu = CargoCommand::Qemu {
            cargoarg,
            example,
            target,
            features: features.clone(),
            mode: BuildMode::Release,
        };
        (globals, cmd_qemu, overwrite)
    });

    build.chain(run).run_and_coalesce()
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
