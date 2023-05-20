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
    argument_parsing::{BuildOrCheck, ExtraArguments, Globals, Package, PackageOpt, TestMetadata},
    cargo_command::{BuildMode, CargoCommand},
};

use log::{debug, error, info};

#[cfg(feature = "rayon")]
use rayon::prelude::*;

fn run_and_convert<'a>(
    (global, command, overwrite): (&Globals, CargoCommand<'a>, bool),
) -> FinalRunResult<'a> {
    // Run the command
    let result = interpret_command(global, &command, overwrite);

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
fn interpret_command(
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
        | CargoCommand::ExampleSize { .. }
        | CargoCommand::Lychee { .. } => {
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
    partial: bool,
) -> Vec<FinalRunResult<'c>> {
    let backend = globals.backend();

    match operation {
        BuildOrCheck::Check => {
            info!("Checking on backend: {backend:?}")
        }
        BuildOrCheck::Build => {
            info!("Building for backend: {backend:?}")
        }
    }

    let runner = package
        .packages()
        .flat_map(|package| {
            let target = backend.to_target();
            let features = package.features(target, backend, partial);
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
    match operation {
        BuildOrCheck::Check => info!("Checking usage examples"),
        BuildOrCheck::Build => info!("Building usage examples"),
    }
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
    examples: Vec<String>,
) -> Vec<FinalRunResult<'c>> {
    let backend = globals.backend();

    match operation {
        BuildOrCheck::Check => {
            info!("Checking examples for backend {backend:?}");
        }
        BuildOrCheck::Build => {
            info!("Building for examples for backend {backend:?}");
        }
    }

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
    partial: bool,
) -> Vec<FinalRunResult<'c>> {
    let backend = globals.backend();
    info!("Running clippy on backend: {backend:?}");

    let runner = package
        .packages()
        .flat_map(|package| {
            let target = backend.to_target();
            let features = package.features(target, backend, partial);
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
    globals: &'c Globals,
    cargoarg: &'c Option<&'c str>,
    arguments: &'c Option<ExtraArguments>,
    check_links: bool,
) -> Vec<FinalRunResult<'c>> {
    let backend = globals.backend();
    info!("Running cargo doc for backend {backend:?}");

    let features = Some(backend.to_target().and_features(backend.to_rtic_feature()));

    let command = CargoCommand::Doc {
        cargoarg,
        features,
        arguments: arguments.clone(),
        deny_warnings: true,
    };

    let mut results = Vec::new();
    let doc = run_and_convert((globals, command, false));
    results.push(doc);
    if results.iter().any(|r| !r.is_success()) {
        return results;
    }

    if check_links {
        let mut links = check_all_api_links(globals);
        results.append(&mut links);
    }
    results
}

/// Run cargo test on the selected package or all packages
///
/// If no package is specified, loop through all packages
pub fn cargo_test<'c>(globals: &Globals, package: &'c PackageOpt) -> Vec<FinalRunResult<'c>> {
    let backend = globals.backend();

    info!("Running cargo test on backend: {backend:?}");

    package
        .packages()
        .map(|p| {
            let meta = TestMetadata::match_package(p, backend);
            (globals, meta, false)
        })
        .run_and_coalesce()
}

/// Run examples
///
/// Supports updating the expected output via the overwrite argument
pub fn qemu_run_examples<'c>(
    globals: &Globals,
    cargoarg: &'c Option<&'c str>,
    examples: Vec<String>,
    overwrite: bool,
) -> Vec<FinalRunResult<'c>> {
    let backend = globals.backend();

    info!("Running QEMU examples for backend: {backend:?}");

    let target = backend.to_target();
    let features = Some(target.and_features(backend.to_rtic_feature()));

    into_iter(examples)
        .flat_map(|example| {
            let target = target.into();
            let dir = Some(PathBuf::from("./rtic"));

            let cmd_build = CargoCommand::ExampleBuild {
                cargoarg: &None,
                example: example.clone(),
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
    examples: Vec<String>,
    arguments: &'c Option<ExtraArguments>,
) -> Vec<FinalRunResult<'c>> {
    let backend = globals.backend();
    info!("Measuring size for backend {backend:?}");

    let target = backend.to_target();
    let features = Some(target.and_features(backend.to_rtic_feature()));

    let runner = into_iter(examples)
        .flat_map(|example| {
            let target = target.into();

            // Make sure the requested example(s) are built
            let cmd_build = CargoCommand::ExampleBuild {
                cargoarg: &Some("--quiet"),
                example: example.clone(),
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
        .stderr(stderr_mode)
        .env_remove("RUST_LOG");

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

fn check_all_api_links(globals: &Globals) -> Vec<FinalRunResult> {
    info!("Checking all API links");
    let runner = Package::all().into_iter().map(|p| {
        let name = p.name().to_string().replace('-', "_");
        let segments = ["target", "doc", name.as_str()];
        let path = PathBuf::from_iter(segments);
        (globals, CargoCommand::Lychee { path }, true)
    });

    runner.run_and_coalesce()
}

/// Use mdbook to build the book
pub fn cargo_book<'c>(
    globals: &'c Globals,
    check_book_links: bool,
    check_api_links: bool,
    output_dir: PathBuf,
    api: Option<PathBuf>,
    arguments: &'c Option<ExtraArguments>,
) -> Vec<FinalRunResult<'c>> {
    info!("Documenting all crates");
    let mut final_results = Vec::new();

    let api_path = if let Some(api) = api {
        if let Err(e) = std::fs::metadata(&api) {
            return vec![FinalRunResult::OtherError(anyhow::anyhow!(
                "Could not find API path: {e}"
            ))];
        }
        api
    } else {
        let doc_command = CargoCommand::Doc {
            cargoarg: &None,
            features: Some(globals.backend().to_rtic_feature().to_string()),
            arguments: None,
            deny_warnings: true,
        };

        final_results.push(run_and_convert((globals, doc_command, true)));
        if final_results.iter().any(|r| !r.is_success()) {
            return final_results;
        }

        if check_api_links {
            let mut links = check_all_api_links(globals);
            final_results.append(&mut links);
            if final_results.iter().any(|r| !r.is_success()) {
                return final_results;
            }
        }

        PathBuf::from_iter(["target", "doc"].into_iter())
    };

    let construct_book = || -> anyhow::Result<Vec<FinalRunResult>> {
        use fs_extra::dir::CopyOptions;

        // ./book-target/
        let book_target = PathBuf::from(output_dir);

        if std::fs::metadata(&book_target)
            .map(|m| !m.is_dir())
            .unwrap_or(false)
        {
            return Err(anyhow::anyhow!(
                "Book target ({}) exists but is not a directory.",
                book_target.display()
            ));
        }

        std::fs::remove_dir_all(&book_target).ok();

        // ./book-target/book
        let mut book_target_book = book_target.clone();
        book_target_book.push("book");

        // ./book-target/book/en
        let mut book_target_en = book_target_book.clone();
        book_target_en.push("en");
        std::fs::create_dir_all(&book_target_en)?;
        let book_target_en = book_target_en.canonicalize()?;

        // ./book-target/api
        let mut book_target_api = book_target.clone();
        book_target_api.push("api");

        info!("Running mdbook");

        let book = run_and_convert((
            globals,
            CargoCommand::Book {
                arguments: arguments.clone(),
                output_path: book_target_en.clone(),
            },
            false,
        ));

        if !book.is_success() {
            return Ok(vec![book]);
        }

        std::fs::create_dir_all(&book_target_book)?;

        debug!("Copying licenses");
        fs_extra::copy_items(
            &["./LICENSE-APACHE", "./LICENSE-CC-BY-SA", "./LICENSE-MIT"],
            &book_target_en,
            &Default::default(),
        )?;

        info!(
            "Copying API docs from {} to {}",
            api_path.display(),
            book_target_api.display()
        );
        fs_extra::copy_items(
            &[api_path],
            book_target_api,
            &CopyOptions::default().overwrite(true).copy_inside(true),
        )?;

        if check_book_links {
            info!("Checking links in the book");

            let last_command = CargoCommand::Lychee {
                path: book_target_en,
            };

            Ok(vec![book, run_and_convert((globals, last_command, true))])
        } else {
            Ok(vec![book])
        }
    };

    let mut construction_result = match construct_book() {
        Ok(res) => res,
        Err(other) => vec![FinalRunResult::OtherError(other)],
    };

    final_results.append(&mut construction_result);
    final_results
}
