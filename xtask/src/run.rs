use std::{
    fs::File,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

mod results;
pub use results::handle_results;

mod data;
use data::*;

mod iter;
use iter::{into_iter, CoalescingRunner};

use crate::{
    argument_parsing::{Backends, BuildOrCheck, ExtraArguments, Globals, PackageOpt, TestMetadata},
    cargo_command::{BuildMode, CargoCommand},
};

use log::{error, info};

#[cfg(feature = "rayon")]
use rayon::prelude::*;

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
            /// Check if `run` was successful.
            /// returns Ok in case the run went as expected,
            /// Err otherwise
            pub fn run_successful(
                run: &RunResult,
                expected_output_file: &str,
            ) -> Result<(), TestRunError> {
                let file = expected_output_file.to_string();

                let expected_output = std::fs::read(expected_output_file)
                    .map(|d| {
                        String::from_utf8(d)
                            .map_err(|_| TestRunError::FileError { file: file.clone() })
                    })
                    .map_err(|_| TestRunError::FileError { file })??;

                let res = if expected_output != run.stdout {
                    Err(TestRunError::FileCmpError {
                        expected: expected_output.clone(),
                        got: run.stdout.clone(),
                    })
                } else if !run.exit_status.success() {
                    Err(TestRunError::CommandError(run.clone()))
                } else {
                    Ok(())
                };

                if res.is_ok() {
                    log::info!("✅ Success.");
                } else {
                    log::error!("❌ Command failed. Run to completion for the summary.");
                }

                res
            }

            let run_file = format!("{example}.run");
            let expected_output_file = ["rtic", "ci", "expected", &run_file]
                .iter()
                .collect::<PathBuf>()
                .into_os_string()
                .into_string()
                .map_err(TestRunError::PathConversionError)?;

            // cargo run <..>
            let cargo_run_result = run_command(command, output_mode, false)?;

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
        | CargoCommand::ExampleSize { .. } => {
            let cargo_result = run_command(command, output_mode, true)?;
            Ok(cargo_result)
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
            into_iter(features).map(move |f| (package, target, f))
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
                    deny_warnings: globals.deny_warnings,
                },
                BuildOrCheck::Build => CargoCommand::Build {
                    cargoarg,
                    package: Some(package.name()),
                    target,
                    features,
                    mode: BuildMode::Release,
                    dir: None,
                    deny_warnings: globals.deny_warnings,
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
    into_iter(&usage_examples)
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
                    deny_warnings: globals.deny_warnings,
                },
                BuildOrCheck::Build => CargoCommand::Build {
                    cargoarg: &None,
                    package: None,
                    target: None,
                    features: None,
                    mode: BuildMode::Release,
                    dir: Some(path.into()),
                    deny_warnings: globals.deny_warnings,
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
    let runner = into_iter(examples).map(|example| {
        let features = Some(backend.to_target().and_features(backend.to_rtic_feature()));

        let command = match operation {
            BuildOrCheck::Check => CargoCommand::ExampleCheck {
                cargoarg,
                example,
                target: Some(backend.to_target()),
                features,
                mode: BuildMode::Release,
                deny_warnings: globals.deny_warnings,
            },
            BuildOrCheck::Build => CargoCommand::ExampleBuild {
                cargoarg,
                example,
                target: Some(backend.to_target()),
                features,
                mode: BuildMode::Release,
                dir: Some(PathBuf::from("./rtic")),
                deny_warnings: globals.deny_warnings,
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
            into_iter(features).map(move |f| (package, target, f))
        })
        .map(move |(package, target, features)| {
            let command = CargoCommand::Clippy {
                cargoarg,
                package: Some(package.name()),
                target: target.into(),
                features,
                deny_warnings: true,
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
        deny_warnings: true,
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
        .map(|p| {
            let meta = TestMetadata::match_package(p, backend);
            (globals, meta, false)
        })
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

    into_iter(examples)
        .flat_map(|example| {
            let target = target.into();
            let dir = Some(PathBuf::from("./rtic"));

            let cmd_build = CargoCommand::ExampleBuild {
                cargoarg: &None,
                example,
                target,
                features: features.clone(),
                mode: BuildMode::Release,
                dir: dir.clone(),
                deny_warnings: globals.deny_warnings,
            };

            let cmd_qemu = CargoCommand::Qemu {
                cargoarg,
                example,
                target,
                features: features.clone(),
                mode: BuildMode::Release,
                dir,
                deny_warnings: globals.deny_warnings,
            };

            into_iter([cmd_build, cmd_qemu])
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

    let runner = into_iter(examples)
        .flat_map(|example| {
            let target = target.into();

            // Make sure the requested example(s) are built
            let cmd_build = CargoCommand::ExampleBuild {
                cargoarg: &Some("--quiet"),
                example,
                target,
                features: features.clone(),
                mode: BuildMode::Release,
                dir: Some(PathBuf::from("./rtic")),
                deny_warnings: globals.deny_warnings,
            };

            let cmd_size = CargoCommand::ExampleSize {
                cargoarg,
                example,
                target,
                features: features.clone(),
                mode: BuildMode::Release,
                arguments: arguments.clone(),
                dir: Some(PathBuf::from("./rtic")),
                deny_warnings: globals.deny_warnings,
            };

            [cmd_build, cmd_size]
        })
        .map(|cmd| (globals, cmd, false));

    runner.run_and_coalesce()
}

fn run_command(
    command: &CargoCommand,
    stderr_mode: OutputMode,
    print_command_success: bool,
) -> anyhow::Result<RunResult> {
    log::info!("👟 {command}");

    let mut process = Command::new(command.executable());

    process
        .args(command.args())
        .stdout(Stdio::piped())
        .stderr(stderr_mode);

    if let Some(dir) = command.chdir() {
        process.current_dir(dir.canonicalize()?);
    }

    if let Some((k, v)) = command.extra_env() {
        process.env(k, v);
    }

    let result = process.output()?;

    let exit_status = result.status;
    let stderr = String::from_utf8(result.stderr).unwrap_or("Not displayable".into());
    let stdout = String::from_utf8(result.stdout).unwrap_or("Not displayable".into());

    if command.print_stdout_intermediate() && exit_status.success() {
        log::info!("\n{}", stdout);
    }

    if print_command_success {
        if exit_status.success() {
            log::info!("✅ Success.")
        } else {
            log::error!("❌ Command failed. Run to completion for the summary.");
        }
    }

    Ok(RunResult {
        exit_status,
        stdout,
        stderr,
    })
}
