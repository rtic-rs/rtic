use crate::{debug, RunResult, Sizearguments, TestRunError};
use core::fmt;
use os_pipe::pipe;
use std::{fs::File, io::Read, process::Command};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuildMode {
    Release,
    Debug,
}

#[derive(Debug)]
pub enum CargoCommand<'a> {
    // For future embedded-ci
    #[allow(dead_code)]
    Run {
        cargoarg: &'a Option<&'a str>,
        example: &'a str,
        target: &'a str,
        features: Option<&'a str>,
        mode: BuildMode,
    },
    Qemu {
        cargoarg: &'a Option<&'a str>,
        example: &'a str,
        target: &'a str,
        features: Option<&'a str>,
        mode: BuildMode,
    },
    ExampleBuild {
        cargoarg: &'a Option<&'a str>,
        example: &'a str,
        target: &'a str,
        features: Option<&'a str>,
        mode: BuildMode,
    },
    ExampleCheck {
        cargoarg: &'a Option<&'a str>,
        example: &'a str,
        target: &'a str,
        features: Option<&'a str>,
        mode: BuildMode,
    },
    Build {
        cargoarg: &'a Option<&'a str>,
        package: Vec<String>,
        target: &'a str,
        features: Option<&'a str>,
        mode: BuildMode,
    },
    Check {
        cargoarg: &'a Option<&'a str>,
        package: Vec<String>,
        target: &'a str,
        features: Option<&'a str>,
        mode: BuildMode,
    },
    Clippy {
        cargoarg: &'a Option<&'a str>,
        package: Vec<String>,
        target: &'a str,
        features: Option<&'a str>,
    },
    Format {
        cargoarg: &'a Option<&'a str>,
        package: Vec<String>,
        check_only: bool,
    },
    Doc {
        cargoarg: &'a Option<&'a str>,
        features: Option<&'a str>,
    },
    Book {
        mdbookarg: &'a Option<&'a str>,
    },
    ExampleSize {
        cargoarg: &'a Option<&'a str>,
        example: &'a str,
        target: &'a str,
        features: Option<&'a str>,
        mode: BuildMode,
        arguments: Option<Sizearguments>,
    },
}

impl<'a> CargoCommand<'a> {
    fn name(&self) -> &str {
        match self {
            CargoCommand::Run { .. } | CargoCommand::Qemu { .. } => "run",
            CargoCommand::ExampleCheck { .. } | CargoCommand::Check { .. } => "check",
            CargoCommand::ExampleBuild { .. } | CargoCommand::Build { .. } => "build",
            CargoCommand::ExampleSize { .. } => "size",
            CargoCommand::Clippy { .. } => "clippy",
            CargoCommand::Format { .. } => "fmt",
            CargoCommand::Doc { .. } => "doc",
            CargoCommand::Book { .. } => "build",
            // TODO
            // CargoCommand::Test { .. } => "test",
        }
    }
    pub fn command(&self) -> &str {
        match self {
            CargoCommand::Run { .. }
            | CargoCommand::Qemu { .. }
            | CargoCommand::ExampleCheck { .. }
            | CargoCommand::Check { .. }
            | CargoCommand::ExampleBuild { .. }
            | CargoCommand::Build { .. }
            | CargoCommand::ExampleSize { .. }
            | CargoCommand::Clippy { .. }
            | CargoCommand::Format { .. }
            | CargoCommand::Doc { .. } => "cargo",
            CargoCommand::Book { .. } => "mdbook",
            // TODO
            // CargoCommand::Test { .. } => "test",
        }
    }

    pub fn args(&self) -> Vec<&str> {
        match self {
            // For future embedded-ci, for now the same as Qemu
            CargoCommand::Run {
                cargoarg,
                example,
                target,
                features,
                mode,
            } => {
                let mut args = vec!["+nightly"];
                if let Some(cargoarg) = cargoarg {
                    args.extend_from_slice(&[cargoarg]);
                }
                args.extend_from_slice(&[self.name(), "--example", example, "--target", target]);

                if let Some(feature) = features {
                    args.extend_from_slice(&["--features", feature]);
                }
                if let Some(flag) = mode.to_flag() {
                    args.push(flag);
                }
                args
            }
            CargoCommand::Qemu {
                cargoarg,
                example,
                target,
                features,
                mode,
            } => {
                let mut args = vec!["+nightly"];
                if let Some(cargoarg) = cargoarg {
                    args.extend_from_slice(&[cargoarg]);
                }
                args.extend_from_slice(&[self.name(), "--example", example, "--target", target]);

                if let Some(feature) = features {
                    args.extend_from_slice(&["--features", feature]);
                }
                if let Some(flag) = mode.to_flag() {
                    args.push(flag);
                }
                args
            }
            CargoCommand::Build {
                cargoarg,
                package,
                target,
                features,
                mode,
            } => {
                let mut args = vec!["+nightly"];
                if let Some(cargoarg) = cargoarg {
                    args.extend_from_slice(&[cargoarg]);
                }

                args.extend_from_slice(&[self.name(), "--target", target]);
                if !package.is_empty() {
                    for package in package {
                        args.extend_from_slice(&["--package", package]);
                    }
                }

                if let Some(feature) = features {
                    args.extend_from_slice(&["--features", feature]);
                }
                if let Some(flag) = mode.to_flag() {
                    args.push(flag);
                }
                args
            }
            CargoCommand::Check {
                cargoarg,
                package,
                target,
                features,
                mode,
            } => {
                let mut args = vec!["+nightly"];
                if let Some(cargoarg) = cargoarg {
                    args.extend_from_slice(&[cargoarg]);
                }
                args.extend_from_slice(&[self.name(), "--target", target]);
                if !package.is_empty() {
                    for package in package {
                        args.extend_from_slice(&["--package", package]);
                    }
                }

                if let Some(feature) = features {
                    args.extend_from_slice(&["--features", feature]);
                }
                if let Some(flag) = mode.to_flag() {
                    args.push(flag);
                }
                args
            }
            CargoCommand::Clippy {
                cargoarg,
                package,
                target,
                features,
            } => {
                let mut args = vec!["+nightly"];
                if let Some(cargoarg) = cargoarg {
                    args.extend_from_slice(&[cargoarg]);
                }

                args.extend_from_slice(&[self.name(), "--target", target]);
                if !package.is_empty() {
                    for package in package {
                        args.extend_from_slice(&["--package", package]);
                    }
                }

                if let Some(feature) = features {
                    args.extend_from_slice(&["--features", feature]);
                }
                args
            }
            CargoCommand::Doc { cargoarg, features } => {
                let mut args = vec!["+nightly"];
                if let Some(cargoarg) = cargoarg {
                    args.extend_from_slice(&[cargoarg]);
                }

                args.extend_from_slice(&[self.name()]);

                if let Some(feature) = features {
                    args.extend_from_slice(&["--features", feature]);
                }
                args
            }
            CargoCommand::Book { mdbookarg } => {
                let mut args = vec![];

                args.extend_from_slice(&[self.name()]);

                if let Some(arg) = mdbookarg {
                    args.extend_from_slice(&[arg]);
                }

                args.extend_from_slice(&["book/en"]);
                args
            }
            CargoCommand::Format {
                cargoarg,
                package,
                check_only,
            } => {
                let mut args = vec!["+nightly", self.name()];
                if let Some(cargoarg) = cargoarg {
                    args.extend_from_slice(&[cargoarg]);
                }

                if !package.is_empty() {
                    for package in package {
                        args.extend_from_slice(&["--package", package]);
                    }
                }
                if *check_only {
                    args.extend_from_slice(&["--check"]);
                }

                args
            }
            CargoCommand::ExampleBuild {
                cargoarg,
                example,
                target,
                features,
                mode,
            } => {
                let mut args = vec!["+nightly"];
                if let Some(cargoarg) = cargoarg {
                    args.extend_from_slice(&[cargoarg]);
                }
                args.extend_from_slice(&[self.name(), "--example", example, "--target", target]);

                if let Some(feature) = features {
                    args.extend_from_slice(&["--features", feature]);
                }
                if let Some(flag) = mode.to_flag() {
                    args.push(flag);
                }
                args
            }
            CargoCommand::ExampleCheck {
                cargoarg,
                example,
                target,
                features,
                mode,
            } => {
                let mut args = vec!["+nightly"];
                if let Some(cargoarg) = cargoarg {
                    args.extend_from_slice(&[cargoarg]);
                }
                args.extend_from_slice(&[self.name(), "--example", example, "--target", target]);

                if let Some(feature) = features {
                    args.extend_from_slice(&["--features", feature]);
                }
                if let Some(flag) = mode.to_flag() {
                    args.push(flag);
                }
                args
            }
            CargoCommand::ExampleSize {
                cargoarg,
                example,
                target,
                features,
                mode,
                arguments,
            } => {
                let mut args = vec!["+nightly"];
                if let Some(cargoarg) = cargoarg {
                    args.extend_from_slice(&[cargoarg]);
                }
                args.extend_from_slice(&[self.name(), "--example", example, "--target", target]);

                if let Some(feature_name) = features {
                    args.extend_from_slice(&["--features", feature_name]);
                }
                if let Some(flag) = mode.to_flag() {
                    args.push(flag);
                }
                if let Some(Sizearguments::Other(arguments)) = arguments {
                    // Arguments to cargo size must be passed after "--"
                    args.extend_from_slice(&["--"]);
                    for arg in arguments {
                        args.extend_from_slice(&[arg.as_str()]);
                    }
                }
                args
            }
        }
    }
}

impl BuildMode {
    #[allow(clippy::wrong_self_convention)]
    pub fn to_flag(&self) -> Option<&str> {
        match self {
            BuildMode::Release => Some("--release"),
            BuildMode::Debug => None,
        }
    }
}

impl fmt::Display for BuildMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cmd = match self {
            BuildMode::Release => "release",
            BuildMode::Debug => "debug",
        };

        write!(f, "{cmd}")
    }
}

pub fn run_command(command: &CargoCommand) -> anyhow::Result<RunResult> {
    let (mut reader, writer) = pipe()?;
    let (mut error_reader, error_writer) = pipe()?;
    debug!("👟 {} {}", command.command(), command.args().join(" "));

    let mut handle = Command::new(command.command())
        .args(command.args())
        .stdout(writer)
        .stderr(error_writer)
        .spawn()?;

    // retrieve output and clean up
    let mut stdout = String::new();
    reader.read_to_string(&mut stdout)?;
    let exit_status = handle.wait()?;

    let mut stderr = String::new();
    error_reader.read_to_string(&mut stderr)?;

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
