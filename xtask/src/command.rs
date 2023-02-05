use crate::{debug, RunResult, Sizearguments, TestRunError};
use core::fmt;
use os_pipe::pipe;
use std::{
    fs::File,
    io::Read,
    process::{Command, Stdio},
};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuildMode {
    Release,
    Debug,
}

#[derive(Debug)]
pub enum CargoCommand<'a> {
    Run {
        cargoarg: &'a Option<&'a str>,
        example: &'a str,
        target: &'a str,
        features: Option<&'a str>,
        mode: BuildMode,
    },
    Build {
        cargoarg: &'a Option<&'a str>,
        example: &'a str,
        target: &'a str,
        features: Option<&'a str>,
        mode: BuildMode,
    },
    BuildAll {
        cargoarg: &'a Option<&'a str>,
        target: &'a str,
        features: Option<&'a str>,
        mode: BuildMode,
    },
    CheckAll {
        cargoarg: &'a Option<&'a str>,
        target: &'a str,
        features: Option<&'a str>,
    },
    Size {
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
            CargoCommand::Run { .. } => "run",
            CargoCommand::Build { .. } => "build",
            CargoCommand::Size { .. } => "size",
            CargoCommand::BuildAll { .. } => "build",
            CargoCommand::CheckAll { .. } => "check",
        }
    }

    pub fn args(&self) -> Vec<&str> {
        match self {
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
            CargoCommand::BuildAll {
                cargoarg,
                target,
                features,
                mode,
            } => {
                let mut args = vec!["+nightly"];
                if let Some(cargoarg) = cargoarg {
                    args.extend_from_slice(&[cargoarg]);
                }
                args.extend_from_slice(&[self.name(), "--examples", "--target", target]);

                if let Some(feature) = features {
                    args.extend_from_slice(&["--features", feature]);
                }
                if let Some(flag) = mode.to_flag() {
                    args.push(flag);
                }
                args
            }
            CargoCommand::CheckAll {
                cargoarg,
                target,
                features,
            } => {
                let mut args = vec!["+nightly"];
                if let Some(cargoarg) = cargoarg {
                    args.extend_from_slice(&[cargoarg]);
                }
                args.extend_from_slice(&[self.name(), "--examples", "--target", target]);

                if let Some(feature) = features {
                    args.extend_from_slice(&["--features", feature]);
                }
                args
            }
            CargoCommand::Build {
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
            CargoCommand::Size {
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

    pub fn command(&self) -> &str {
        "cargo"
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
    debug!("👟 {} {}", command.command(), command.args().join(" "));

    let mut handle = Command::new(command.command())
        .args(command.args())
        .stdout(writer)
        // Throw away stderr, TODO
        .stderr(Stdio::null())
        .spawn()?;

    // retrieve output and clean up
    let mut output = String::new();
    reader.read_to_string(&mut output)?;
    let exit_status = handle.wait()?;

    Ok(RunResult {
        exit_status,
        output,
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

    if expected_output != run.output {
        Err(TestRunError::FileCmpError {
            expected: expected_output.clone(),
            got: run.output.clone(),
        })
    } else if !run.exit_status.success() {
        Err(TestRunError::CommandError(run.clone()))
    } else {
        Ok(())
    }
}
