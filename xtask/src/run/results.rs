use log::{error, info, log, Level};

use crate::{argument_parsing::Globals, cargo_command::CargoCommand};

use super::data::{FinalRunResult, RunResult, TestRunError};

const TARGET: &str = "xtask::results";

/// Check if `run` was successful.
/// returns Ok in case the run went as expected,
/// Err otherwise
pub fn run_successful(run: &RunResult, expected_output_file: &str) -> Result<(), TestRunError> {
    let file = expected_output_file.to_string();

    let expected_output = std::fs::read(expected_output_file)
        .map(|d| String::from_utf8(d).map_err(|_| TestRunError::FileError { file: file.clone() }))
        .map_err(|_| TestRunError::FileError { file })??;

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

pub fn handle_results(globals: &Globals, results: Vec<FinalRunResult>) -> Result<(), ()> {
    let errors = results.iter().filter_map(|r| {
        if let FinalRunResult::Failed(c, r) = r {
            Some((c, &r.stdout, &r.stderr))
        } else {
            None
        }
    });

    let successes = results.iter().filter_map(|r| {
        if let FinalRunResult::Success(c, r) = r {
            Some((c, &r.stdout, &r.stderr))
        } else {
            None
        }
    });

    let command_errors = results.iter().filter_map(|r| {
        if let FinalRunResult::CommandError(c, e) = r {
            Some((c, e))
        } else {
            None
        }
    });

    let log_stdout_stderr = |level: Level| {
        move |(cmd, stdout, stderr): (&CargoCommand, &String, &String)| {
            let cmd = cmd.as_cmd_string();
            if !stdout.is_empty() && !stderr.is_empty() {
                log!(
                    target: TARGET,
                    level,
                    "\n{cmd}\nStdout:\n{stdout}\nStderr:\n{stderr}"
                );
            } else if !stdout.is_empty() {
                log!(
                    target: TARGET,
                    level,
                    "\n{cmd}\nStdout:\n{}",
                    stdout.trim_end()
                );
            } else if !stderr.is_empty() {
                log!(
                    target: TARGET,
                    level,
                    "\n{cmd}\nStderr:\n{}",
                    stderr.trim_end()
                );
            }
        }
    };

    successes.for_each(|(cmd, stdout, stderr)| {
        if globals.verbose > 0 {
            info!(
                target: TARGET,
                "✅ Success: {cmd}\n    {}",
                cmd.as_cmd_string()
            );
        } else {
            info!(target: TARGET, "✅ Success: {cmd}");
        }

        log_stdout_stderr(Level::Debug)((cmd, stdout, stderr));
    });

    errors.clone().for_each(|(cmd, stdout, stderr)| {
        error!(
            target: TARGET,
            "❌ Failed: {cmd}\n    {}",
            cmd.as_cmd_string()
        );
        log_stdout_stderr(Level::Error)((cmd, stdout, stderr));
    });

    command_errors.clone().for_each(|(cmd, error)| {
        error!(
            target: TARGET,
            "❌ Failed: {cmd}\n    {}\n{error}",
            cmd.as_cmd_string()
        )
    });

    let ecount = errors.count() + command_errors.count();
    if ecount != 0 {
        error!(target: TARGET, "{ecount} commands failed.");
        Err(())
    } else {
        info!(target: TARGET, "🚀🚀🚀 All tasks succeeded 🚀🚀🚀");
        Ok(())
    }
}
