use crate::RunResult;
use core::fmt;
use os_pipe::pipe;
use std::{fs::File, io::Read, path::Path, process::Command};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuildMode {
    Release,
    Debug,
}

pub enum CargoCommand<'a> {
    Run {
        example: &'a str,
        target: &'a str,
        features: Option<&'a str>,
        mode: BuildMode,
    },
    Build {
        example: &'a str,
        target: &'a str,
        features: Option<&'a str>,
        mode: BuildMode,
    },
    Objcopy {
        example: &'a str,
        target: &'a str,
        features: Option<&'a str>,
        ihex: &'a str,
    },
    Size {
        example_paths: Vec<&'a Path>,
    },
    Clean,
}

impl<'a> CargoCommand<'a> {
    fn name(&self) -> &str {
        match self {
            CargoCommand::Run { .. } => "run",
            CargoCommand::Size { example_paths: _ } => "rust-size",
            CargoCommand::Clean => "clean",
            CargoCommand::Build { .. } => "build",
            CargoCommand::Objcopy { .. } => "objcopy",
        }
    }

    pub fn args(&self) -> Vec<&str> {
        match self {
            CargoCommand::Run {
                example,
                target,
                features,
                mode,
            }
            | CargoCommand::Build {
                example,
                target,
                features,
                mode,
            } => {
                let mut args = vec![self.name(), "--example", example, "--target", target];

                if let Some(feature_name) = features {
                    args.extend_from_slice(&["--features", feature_name]);
                }
                if let Some(flag) = mode.to_flag() {
                    args.push(flag);
                }
                args
            }
            CargoCommand::Size { example_paths } => {
                example_paths.iter().map(|p| p.to_str().unwrap()).collect()
            }
            CargoCommand::Clean => vec!["clean"],
            CargoCommand::Objcopy {
                example,
                target,
                features,
                ihex,
            } => {
                let mut args = vec![self.name(), "--example", example, "--target", target];

                if let Some(feature_name) = features {
                    args.extend_from_slice(&["--features", feature_name]);
                }

                // this always needs to go at the end
                args.extend_from_slice(&["--", "-O", "ihex", ihex]);
                args
            }
        }
    }

    pub fn command(&self) -> &str {
        match self {
            // we need to cheat a little here:
            // `cargo size` can't be ran on multiple files, so we're using `rust-size` instead –
            // which isn't a command that starts wizh `cargo`. So we're sneakily swapping them out :)
            CargoCommand::Size { .. } => "rust-size",
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
pub fn run_successful(run: &RunResult, expected_output_file: String) -> anyhow::Result<()> {
    let mut file_handle = File::open(expected_output_file)?;
    let mut expected_output = String::new();
    file_handle.read_to_string(&mut expected_output)?;
    if expected_output == run.output && run.exit_status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Run failed with exit status {}: {}",
            run.exit_status,
            run.output
        ))
    }
}
