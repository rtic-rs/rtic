use std::{
    fs::File,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

mod results;
pub use results::handle_results;

mod data;
pub use data::*;

mod iter;
use iter::CoalescingRunner;

use crate::{
    argument_parsing::{
        BuildOrCheck, ExampleArgs, FormatOpt, Globals, PackageOpt, Platform, RticBackend,
    },
    cargo_command::{BuildMode, CargoCommand},
};

use log::{error, info};

fn run_and_convert((global, command, overwrite): (&Globals, CargoCommand, bool)) -> FinalRunResult {
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

    match command {
        CargoCommand::Qemu {
            platform, example, ..
        }
        | CargoCommand::ExampleSize {
            platform, example, ..
        }
        | CargoCommand::Run {
            platform, example, ..
        } => {
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

            let platform_name = platform.name();
            let run_file = if let CargoCommand::ExampleSize { .. } = *command {
                format!("{example}.size")
            } else {
                format!("{example}.run")
            };

            let expected_output_file = ["ci", "expected", &platform_name, &run_file]
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
        | CargoCommand::Book { .. } => {
            let cargo_result = run_command(command, output_mode, true)?;
            Ok(cargo_result)
        }
    }
}

/// Cargo command to either build or check
pub fn cargo(
    globals: &Globals,
    operation: BuildOrCheck,
    cargoarg: Option<String>,
    package: PackageOpt,
) -> Vec<FinalRunResult> {
    info!("Building packages");
    let runner = package.packages().map(move |package| {
        let target = package.target();
        let features = package.features(true);
        let mode = BuildMode::Release;
        let command = match operation {
            BuildOrCheck::Check => CargoCommand::Check {
                cargoarg: cargoarg.clone(),
                package: Some(package),
                target,
                features,
                mode,
                dir: None,
                deny_warnings: globals.deny_warnings,
            },
            BuildOrCheck::Build => CargoCommand::Build {
                cargoarg: cargoarg.clone(),
                package: Some(package),
                target,
                features,
                mode,
                dir: None,
                deny_warnings: globals.deny_warnings,
            },
        };

        (globals, command, false)
    });

    runner.run_and_coalesce()
}

/// Cargo command to either build or check all examples
///
/// The examples are in examples/<platform>/examples
pub fn cargo_example(
    globals: &Globals,
    operation: BuildOrCheck,
    cargoarg: Option<String>,
    platforms: Vec<Platform>,
    examples: ExampleArgs,
) -> Vec<FinalRunResult> {
    let runner = platforms.into_iter().flat_map(|platform| {
        info!("Checking examples for platform {platform:?}");
        let path = format!("examples/{}", platform.name());
        let dir = Some(PathBuf::from(path));
        let mode = BuildMode::Release;
        let cargoarg = cargoarg.clone();

        let examples = examples
            .get_examples(&platform)
            .unwrap_or_else(|_| panic!("Failed to get examples for platform {platform:?}"));

        examples.into_iter().map(move |example| {
            let dir = dir.clone();

            let command = match operation {
                BuildOrCheck::Check => CargoCommand::ExampleCheck {
                    cargoarg: cargoarg.clone(),
                    platform,
                    example,
                    mode,
                    dir,
                    deny_warnings: globals.deny_warnings,
                },
                BuildOrCheck::Build => CargoCommand::ExampleBuild {
                    cargoarg: cargoarg.clone(),
                    example,
                    platform,
                    mode,
                    dir,
                    deny_warnings: globals.deny_warnings,
                },
            };

            (globals, command, false)
        })
    });

    runner.run_and_coalesce()
}

/// Run cargo clippy on selected package
pub fn cargo_clippy(
    globals: &Globals,
    cargoarg: Option<String>,
    package: PackageOpt,
) -> Vec<FinalRunResult> {
    info!("Running clippy for {package:?}");
    let runner = package.packages().map(move |package| {
        let target = package.target();
        let features = package.features(true);
        let command = CargoCommand::Clippy {
            cargoarg: cargoarg.clone(),
            package: Some(package),
            target,
            features,
            deny_warnings: true,
        };

        (globals, command, false)
    });

    runner.run_and_coalesce()
}

/// Run cargo fmt on selected package
pub fn cargo_format(
    globals: &Globals,
    cargoarg: Option<String>,
    formatopts: &FormatOpt,
) -> anyhow::Result<Vec<FinalRunResult>> {
    // TODO: activate & format all examples
    #[expect(unused)]
    fn find_tomls(output: &mut Vec<PathBuf>, current: PathBuf) -> anyhow::Result<()> {
        for entry in std::fs::read_dir(current)? {
            let entry = entry?;

            if entry.file_type()?.is_dir() && entry.file_name() != "target" {
                find_tomls(output, entry.path())?;
            } else if entry.file_name() == "Cargo.toml" {
                output.push(entry.path());
            }
        }

        Ok(())
    }

    // Start off by just formatting all workspace packages.
    let output = vec![PathBuf::from("./Cargo.toml")];

    let runner = output.into_iter().map(|manifest| {
        (
            globals,
            CargoCommand::Format {
                cargoarg: cargoarg.clone(),
                manifest,
                check_only: formatopts.check,
            },
            false,
        )
    });

    Ok(runner.run_and_coalesce())
}

/// Run cargo doc
pub fn cargo_doc(
    globals: &Globals,
    cargoarg: Option<String>,
    arguments: &[String],
) -> Vec<FinalRunResult> {
    let extra_doc_features = [
        "rtic-monotonics/cortex-m-systick",
        "rtic-monotonics/rp2040",
        "rtic-monotonics/nrf52840",
        "imxrt-ral/imxrt1011",
        "rtic-monotonics/imxrt_gpt1",
        "rtic-monotonics/imxrt_gpt2",
        "stm32-metapac/stm32h725ag",
        "rtic-monotonics/stm32_tim2",
        "rtic-monotonics/stm32_tim3",
        "rtic-monotonics/stm32_tim4",
        "rtic-monotonics/stm32_tim5",
        "rtic-monotonics/stm32_tim15",
    ];

    // TODO: pick a sensible default
    let backend = RticBackend::Thumbv7;

    let features = Some(format!(
        "{},{}",
        backend.rtic_feature(),
        extra_doc_features.join(",")
    ));

    let command = CargoCommand::Doc {
        cargoarg,
        features,
        arguments: arguments.to_owned(),
        deny_warnings: true,
    };

    vec![run_and_convert((globals, command, false))]
}

/// Run cargo test on the selected package or all packages
///
/// If no package is specified, loop through all packages
pub fn cargo_test(globals: &Globals, opts: PackageOpt, loom: bool) -> Vec<FinalRunResult> {
    info!("Running cargo test");
    opts.packages()
        .map(|p| {
            let meta = p.test_command(loom);
            (globals, meta, false)
        })
        .run_and_coalesce()
}

/// Use mdbook to build the book
pub fn cargo_book(globals: &Globals, arguments: &[String]) -> Vec<FinalRunResult> {
    info!("Running mdbook");
    vec![run_and_convert((
        globals,
        CargoCommand::Book {
            arguments: arguments.to_owned(),
        },
        false,
    ))]
}

/// Run examples
///
/// Supports updating the expected output via the overwrite argument
///
/// The examples are in examples/<platform>/examples
pub fn qemu_run_examples(
    globals: &Globals,
    cargoarg: Option<String>,
    platform: Platform,
    examples: Vec<String>,
    overwrite: bool,
) -> Vec<FinalRunResult> {
    info!("QEMU run for platform: {platform:?}");
    examples
        .into_iter()
        .flat_map(|example| {
            let path = format!("examples/{}", platform.name());
            let dir = Some(PathBuf::from(path));
            let mode = BuildMode::Release;

            let cmd_build = CargoCommand::ExampleBuild {
                cargoarg: None,
                example: example.clone(),
                platform,
                mode,
                dir: dir.clone(),
                deny_warnings: globals.deny_warnings,
            };

            let cmd_qemu = CargoCommand::Qemu {
                cargoarg: cargoarg.clone(),
                platform,
                example,
                mode,
                dir,
                deny_warnings: globals.deny_warnings,
            };

            [cmd_build, cmd_qemu].into_iter()
        })
        .map(|cmd| (globals, cmd, overwrite))
        .run_and_coalesce()
}

/// Check the binary sizes of examples
pub fn build_and_check_size(
    globals: &Globals,
    cargoarg: Option<String>,
    platforms: Vec<Platform>,
    examples: &ExampleArgs,
    overwrite: bool,
    arguments: &[String],
) -> Vec<FinalRunResult> {
    let runner = platforms.into_iter().flat_map(|platform| {
        info!("Measuring for platform: {platform:?}");

        let examples = examples
            .get_examples(&platform)
            .unwrap_or_else(|_| panic!("Failed to get examples for platform {platform:?}"));
        let cargoarg = cargoarg.clone();

        examples
            .into_iter()
            .flat_map(move |example| {
                let path = format!("examples/{}", platform.name());
                let dir = Some(PathBuf::from(path));
                let mode = BuildMode::Release;
                let cargoarg = cargoarg.clone();

                // Make sure the requested example(s) are built
                let cmd_build = CargoCommand::ExampleBuild {
                    cargoarg: Some("--quiet".to_string()),
                    example: example.clone(),
                    platform,
                    mode,
                    dir: dir.clone(),
                    deny_warnings: globals.deny_warnings,
                };

                let cmd_size = CargoCommand::ExampleSize {
                    cargoarg,
                    platform,
                    example,
                    mode,
                    arguments: arguments.to_owned(),
                    dir,
                    deny_warnings: globals.deny_warnings,
                };

                [cmd_build, cmd_size]
            })
            .map(|cmd| (globals, cmd, overwrite))
    });

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
