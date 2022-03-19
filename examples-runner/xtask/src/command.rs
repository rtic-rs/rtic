use crate::{RunResult, TestRunError};
use clap::ArgEnum;
use core::fmt;
use os_pipe::pipe;
use std::{fs::File, io::Read, process::Command};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuildMode {
    Release,
    Debug,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
pub enum Runner {
    Qemu,
    EmbeddedCi,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ArgEnum)]
pub enum CoreRun {
    CortexM0,
    CortexM3,
    CortexM4,
    CortexM7,
}

#[derive(Debug)]
pub enum CargoCommand<'a> {
    Run {
        example: &'a str,
        target: &'a str,
        features: Option<&'a str>,
        mode: BuildMode,
        runner: Runner,
        core_runner: Option<CoreRun>,
    },
    BuildAll {
        target: &'a str,
        features: Option<&'a str>,
        mode: BuildMode,
    },
    // Size {
    //     example_paths: Vec<&'a Path>,
    // },
    // Clean,
}

impl<'a> CargoCommand<'a> {
    fn name(&self) -> &str {
        match self {
            CargoCommand::Run { .. } => "run",
            // CargoCommand::Size { example_paths: _ } => "rust-size",
            CargoCommand::BuildAll { .. } => "build",
        }
    }

    pub fn args(&self) -> Vec<&str> {
        match self {
            CargoCommand::Run {
                example,
                target,
                features,
                mode,
                runner,
                core_runner,
            } => match runner {
                Runner::Qemu => {
                    let mut args = vec![self.name(), "--bin", example, "--target", target];

                    if let Some(feature_name) = features {
                        args.extend_from_slice(&["--features", feature_name]);
                    }
                    if let Some(flag) = mode.to_flag() {
                        args.push(flag);
                    }
                    args
                }
                Runner::EmbeddedCi => {
                    let mut args = vec![];
                    match core_runner {
                        Some(CoreRun::CortexM0) => {
                            args.extend_from_slice(&["--cores", "cortexm0plus"])
                        }
                        Some(CoreRun::CortexM3) => args.extend_from_slice(&["--cores", "cortexm3"]),
                        Some(CoreRun::CortexM4) => args.extend_from_slice(&["--cores", "cortexm4"]),
                        Some(CoreRun::CortexM7) => args.extend_from_slice(&["--cores", "cortexm7"]),
                        None => {
                            if target.contains("thumbv6") {
                                args.extend_from_slice(&["--cores", "cortexm0plus"])
                            } else if target.contains("thumbv7") {
                                args.extend_from_slice(&["--cores", "cortexm3,cortexm4,cortexm7"])
                            } else {
                                panic!("Unknown target: {}", target);
                            }
                        }
                    }
                    let s = Box::new(format!("target/{target}/{mode}/{example}"));
                    let s = s.into_boxed_str();
                    let s: &'static str = Box::leak(s);

                    args.push(s);

                    args
                }
            },
            CargoCommand::BuildAll {
                target,
                features,
                mode,
            } => {
                let mut args = vec![self.name(), "--bins", "--target", target];

                if let Some(feature_name) = features {
                    args.extend_from_slice(&["--features", feature_name]);
                }
                if let Some(flag) = mode.to_flag() {
                    args.push(flag);
                }
                args
            } // CargoCommand::Size { example_paths } => {
              //     example_paths.iter().map(|p| p.to_str().unwrap()).collect()
              // }
        }
    }

    pub fn command(&self) -> &str {
        match self {
            CargoCommand::Run {
                example: _,
                target: _,
                features: _,
                mode: _,
                runner,
                core_runner: _,
            } => match runner {
                Runner::Qemu => "cargo",
                Runner::EmbeddedCi => "embedded-ci-client",
            },
            // we need to cheat a little here:
            // `cargo size` can't be ran on multiple files, so we're using `rust-size` instead –
            // which isn't a command that starts wizh `cargo`. So we're sneakily swapping them out :)
            // CargoCommand::Size { .. } => "rust-size",
            _ => "cargo",
        }
    }
}

impl BuildMode {
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

        write!(f, "{}", cmd)
    }
}

pub fn run_command(command: &CargoCommand) -> anyhow::Result<RunResult> {
    let (mut reader, writer) = pipe()?;
    println!("👟 {} {}", command.command(), command.args().join(" "));

    let mut handle = Command::new(command.command())
        .args(command.args())
        .stdout(writer)
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

/// Check if `run` was sucessful.
/// returns Ok in case the run went as expected,
/// Err otherwise
pub fn run_successful(run: &RunResult, expected_output_file: String) -> Result<(), TestRunError> {
    let mut file_handle =
        File::open(expected_output_file.clone()).map_err(|_| TestRunError::FileError {
            file: expected_output_file.clone(),
        })?;
    let mut expected_output = String::new();
    file_handle
        .read_to_string(&mut expected_output)
        .map_err(|_| TestRunError::FileError {
            file: expected_output_file.clone(),
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
