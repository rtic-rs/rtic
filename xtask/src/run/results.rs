use log::{error, info, log, Level};

use crate::{argument_parsing::Globals, cargo_command::CargoCommand};

use super::data::FinalRunResult;

const TARGET: &str = "xtask::results";

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
