use std::{
    ffi::OsString,
    process::{ExitStatus, Stdio},
};

use diffy::{create_patch, PatchFormatter};

use crate::cargo_command::CargoCommand;

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

#[derive(Debug, Clone)]
pub struct RunResult {
    pub exit_status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug)]
pub enum FinalRunResult<'c> {
    Success(CargoCommand<'c>, RunResult),
    Failed(CargoCommand<'c>, RunResult),
    CommandError(CargoCommand<'c>, anyhow::Error),
}

#[derive(Debug)]
pub enum TestRunError {
    FileCmpError {
        expected: String,
        got: String,
    },
    FileError {
        file: String,
    },
    PathConversionError(OsString),
    CommandError(RunResult),
    #[allow(dead_code)]
    IncompatibleCommand,
}

impl core::fmt::Display for TestRunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestRunError::FileCmpError { expected, got } => {
                let patch = create_patch(expected, got);
                writeln!(f, "Differing output in files.\n")?;
                let pf = PatchFormatter::new().with_color();
                writeln!(f, "{}", pf.fmt_patch(&patch))?;
                write!(
                    f,
                    "See flag --overwrite-expected to create/update expected output."
                )
            }
            TestRunError::FileError { file } => {
                write!(f, "File error on: {file}\nSee flag --overwrite-expected to create/update expected output.")
            }
            TestRunError::CommandError(e) => {
                write!(
                    f,
                    "Command failed with exit status {}: {} {}",
                    e.exit_status, e.stdout, e.stderr
                )
            }
            TestRunError::PathConversionError(p) => {
                write!(f, "Can't convert path from `OsString` to `String`: {p:?}")
            }
            TestRunError::IncompatibleCommand => {
                write!(f, "Can't run that command in this context")
            }
        }
    }
}

impl std::error::Error for TestRunError {}
