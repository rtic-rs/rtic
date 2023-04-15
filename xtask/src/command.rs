use log::{error, info, Level};

use crate::{
    argument_parsing::Globals, cargo_commands::FinalRunResult, ExtraArguments, Package, RunResult,
    Target, TestRunError,
};
use core::fmt;
use std::{
    fs::File,
    io::Read,
    path::PathBuf,
    process::{Command, Stdio},
};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuildMode {
    Release,
    Debug,
}

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

#[derive(Debug)]
pub enum CargoCommand<'a> {
    // For future embedded-ci
    #[allow(dead_code)]
    Run {
        cargoarg: &'a Option<&'a str>,
        example: &'a str,
        target: Target<'a>,
        features: Option<String>,
        mode: BuildMode,
    },
    Qemu {
        cargoarg: &'a Option<&'a str>,
        example: &'a str,
        target: Target<'a>,
        features: Option<String>,
        mode: BuildMode,
    },
    ExampleBuild {
        cargoarg: &'a Option<&'a str>,
        example: &'a str,
        target: Target<'a>,
        features: Option<String>,
        mode: BuildMode,
    },
    ExampleCheck {
        cargoarg: &'a Option<&'a str>,
        example: &'a str,
        target: Target<'a>,
        features: Option<String>,
        mode: BuildMode,
    },
    Build {
        cargoarg: &'a Option<&'a str>,
        package: Option<Package>,
        target: Target<'a>,
        features: Option<String>,
        mode: BuildMode,
    },
    Check {
        cargoarg: &'a Option<&'a str>,
        package: Option<Package>,
        target: Target<'a>,
        features: Option<String>,
        mode: BuildMode,
    },
    Clippy {
        cargoarg: &'a Option<&'a str>,
        package: Option<Package>,
        target: Target<'a>,
        features: Option<String>,
    },
    Format {
        cargoarg: &'a Option<&'a str>,
        package: Option<Package>,
        check_only: bool,
    },
    Doc {
        cargoarg: &'a Option<&'a str>,
        features: Option<String>,
        arguments: Option<ExtraArguments>,
    },
    Test {
        package: Option<Package>,
        features: Option<String>,
        test: Option<String>,
    },
    Book {
        arguments: Option<ExtraArguments>,
    },
    ExampleSize {
        cargoarg: &'a Option<&'a str>,
        example: &'a str,
        target: Target<'a>,
        features: Option<String>,
        mode: BuildMode,
        arguments: Option<ExtraArguments>,
    },
    CheckInDir {
        mode: BuildMode,
        dir: PathBuf,
    },
    BuildInDir {
        mode: BuildMode,
        dir: PathBuf,
    },
}

impl core::fmt::Display for CargoCommand<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let p = |p: &Option<Package>| {
            if let Some(package) = p {
                format!("package {package}")
            } else {
                format!("default package")
            }
        };

        let feat = |f: &Option<String>| {
            if let Some(features) = f {
                format!("\"{features}\"")
            } else {
                format!("no features")
            }
        };

        let carg = |f: &&Option<&str>| {
            if let Some(cargoarg) = f {
                format!("{cargoarg}")
            } else {
                format!("no cargo args")
            }
        };

        let details = |target: &Target,
                       mode: &BuildMode,
                       features: &Option<String>,
                       cargoarg: &&Option<&str>| {
            let feat = feat(features);
            let carg = carg(cargoarg);
            if cargoarg.is_some() {
                format!("({target}, {mode}, {feat}, {carg})")
            } else {
                format!("({target}, {mode}, {feat})")
            }
        };

        match self {
            CargoCommand::Run {
                cargoarg,
                example,
                target,
                features,
                mode,
            } => write!(
                f,
                "Run example {example} {}",
                details(target, mode, features, cargoarg)
            ),
            CargoCommand::Qemu {
                cargoarg,
                example,
                target,
                features,
                mode,
            } => write!(
                f,
                "Run example {example} in QEMU {}",
                details(target, mode, features, cargoarg)
            ),
            CargoCommand::ExampleBuild {
                cargoarg,
                example,
                target,
                features,
                mode,
            } => write!(
                f,
                "Build example {example} {}",
                details(target, mode, features, cargoarg)
            ),
            CargoCommand::ExampleCheck {
                cargoarg,
                example,
                target,
                features,
                mode,
            } => write!(
                f,
                "Check example {example} {}",
                details(target, mode, features, cargoarg)
            ),
            CargoCommand::Build {
                cargoarg,
                package,
                target,
                features,
                mode,
            } => {
                let package = p(package);
                write!(
                    f,
                    "Build {package} {}",
                    details(target, mode, features, cargoarg)
                )
            }
            CargoCommand::BuildInDir { mode, dir } => {
                let dir = dir.as_os_str().to_str().unwrap_or("Not displayable");
                write!(f, "Build {dir} ({mode})")
            }
            CargoCommand::Check {
                cargoarg,
                package,
                target,
                features,
                mode,
            } => {
                let package = p(package);
                write!(
                    f,
                    "Check {package} {}",
                    details(target, mode, features, cargoarg)
                )
            }
            CargoCommand::CheckInDir { mode, dir } => {
                let dir = dir.as_os_str().to_str().unwrap_or("Not displayable");
                write!(f, "Check {dir} ({mode})")
            }
            CargoCommand::Clippy {
                cargoarg,
                package,
                target,
                features,
            } => {
                let package = p(package);
                let features = feat(features);
                let carg = carg(cargoarg);
                if cargoarg.is_some() {
                    write!(f, "Clippy {package} ({target}, {features}, {carg})")
                } else {
                    write!(f, "Clippy {package} ({target}, {features})")
                }
            }
            CargoCommand::Format {
                cargoarg,
                package,
                check_only,
            } => {
                let package = p(package);
                let carg = carg(cargoarg);

                let carg = if cargoarg.is_some() {
                    format!("(cargo args: {carg})")
                } else {
                    format!("")
                };

                if *check_only {
                    write!(f, "Check format for {package} {carg}")
                } else {
                    write!(f, "Format {package} {carg}")
                }
            }
            CargoCommand::Doc {
                cargoarg,
                features,
                arguments,
            } => {
                let feat = feat(features);
                let carg = carg(cargoarg);
                let arguments = arguments
                    .clone()
                    .map(|a| format!("{a}"))
                    .unwrap_or_else(|| "no extra arguments".into());
                if cargoarg.is_some() {
                    write!(f, "Document ({feat}, {carg}, {arguments})")
                } else {
                    write!(f, "Document ({feat}, {arguments})")
                }
            }
            CargoCommand::Test {
                package,
                features,
                test,
            } => {
                let p = p(package);
                let test = test
                    .clone()
                    .map(|t| format!("test {t}"))
                    .unwrap_or("all tests".into());
                let feat = feat(features);
                write!(f, "Run {test} in {p} (features: {feat})")
            }
            CargoCommand::Book { arguments: _ } => write!(f, "Build the book"),
            CargoCommand::ExampleSize {
                cargoarg,
                example,
                target,
                features,
                mode,
                arguments: _,
            } => {
                write!(
                    f,
                    "Compute size of example {example} {}",
                    details(target, mode, features, cargoarg)
                )
            }
        }
    }
}

impl<'a> CargoCommand<'a> {
    pub fn as_cmd_string(&self) -> String {
        let executable = self.executable();
        let args = self.args().join(" ");
        format!("{executable} {args}")
    }

    fn command(&self) -> &'static str {
        match self {
            CargoCommand::Run { .. } | CargoCommand::Qemu { .. } => "run",
            CargoCommand::ExampleCheck { .. }
            | CargoCommand::Check { .. }
            | CargoCommand::CheckInDir { .. } => "check",
            CargoCommand::ExampleBuild { .. }
            | CargoCommand::Build { .. }
            | CargoCommand::BuildInDir { .. } => "build",
            CargoCommand::ExampleSize { .. } => "size",
            CargoCommand::Clippy { .. } => "clippy",
            CargoCommand::Format { .. } => "fmt",
            CargoCommand::Doc { .. } => "doc",
            CargoCommand::Book { .. } => "build",
            CargoCommand::Test { .. } => "test",
        }
    }
    pub fn executable(&self) -> &'static str {
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
            | CargoCommand::Test { .. }
            | CargoCommand::Doc { .. }
            | CargoCommand::CheckInDir { .. }
            | CargoCommand::BuildInDir { .. } => "cargo",
            CargoCommand::Book { .. } => "mdbook",
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

                args.extend_from_slice(&[
                    self.command(),
                    "--example",
                    example,
                    "--target",
                    target.triple(),
                ]);

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

                // We need to be in the `rtic` directory to pick up
                // the correct .cargo/config.toml file
                args.extend_from_slice(&["-Z", "unstable-options", "-C", "rtic"]);

                args.extend_from_slice(&[
                    self.command(),
                    "--example",
                    example,
                    "--target",
                    target.triple(),
                ]);

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

                args.extend_from_slice(&[self.command(), "--target", target.triple()]);

                if let Some(package) = package {
                    args.extend_from_slice(&["--package", package.name()]);
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
                target: _,
                features,
                mode,
            } => {
                let mut args = vec!["+nightly"];
                if let Some(cargoarg) = cargoarg {
                    args.extend_from_slice(&[cargoarg]);
                }
                args.extend_from_slice(&[self.command()]);

                if let Some(package) = package {
                    args.extend_from_slice(&["--package", package.name()]);
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
                target: _,
                features,
            } => {
                let mut args = vec!["+nightly"];
                if let Some(cargoarg) = cargoarg {
                    args.extend_from_slice(&[cargoarg]);
                }

                args.extend_from_slice(&[self.command()]);

                if let Some(package) = package {
                    args.extend_from_slice(&["--package", package.name()]);
                }

                if let Some(feature) = features {
                    args.extend_from_slice(&["--features", feature]);
                }
                args
            }
            CargoCommand::Doc {
                cargoarg,
                features,
                arguments,
            } => {
                let mut args = vec!["+nightly"];
                if let Some(cargoarg) = cargoarg {
                    args.extend_from_slice(&[cargoarg]);
                }

                args.extend_from_slice(&[self.command()]);

                if let Some(feature) = features {
                    args.extend_from_slice(&["--features", feature]);
                }
                if let Some(ExtraArguments::Other(arguments)) = arguments {
                    for arg in arguments {
                        args.extend_from_slice(&[arg.as_str()]);
                    }
                }
                args
            }
            CargoCommand::Test {
                package,
                features,
                test,
            } => {
                let mut args = vec!["+nightly"];
                args.extend_from_slice(&[self.command()]);

                if let Some(package) = package {
                    args.extend_from_slice(&["--package", package.name()]);
                }

                if let Some(feature) = features {
                    args.extend_from_slice(&["--features", feature]);
                }
                if let Some(test) = test {
                    args.extend_from_slice(&["--test", test]);
                }
                args
            }
            CargoCommand::Book { arguments } => {
                let mut args = vec![];

                if let Some(ExtraArguments::Other(arguments)) = arguments {
                    for arg in arguments {
                        args.extend_from_slice(&[arg.as_str()]);
                    }
                } else {
                    // If no argument given, run mdbook build
                    // with default path to book
                    args.extend_from_slice(&[self.command()]);
                    args.extend_from_slice(&["book/en"]);
                }
                args
            }
            CargoCommand::Format {
                cargoarg,
                package,
                check_only,
            } => {
                let mut args = vec!["+nightly", self.command()];
                if let Some(cargoarg) = cargoarg {
                    args.extend_from_slice(&[cargoarg]);
                }

                if let Some(package) = package {
                    args.extend_from_slice(&["--package", package.name()]);
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

                // We need to be in the `rtic` directory to pick up
                // the correct .cargo/config.toml file
                args.extend_from_slice(&["-Z", "unstable-options", "-C", "rtic"]);

                args.extend_from_slice(&[
                    self.command(),
                    "--example",
                    example,
                    "--target",
                    target.triple(),
                ]);

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
                args.extend_from_slice(&[
                    self.command(),
                    "--example",
                    example,
                    "--target",
                    target.triple(),
                ]);

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

                // We need to be in the `rtic` directory to pick up
                // the correct .cargo/config.toml file
                args.extend_from_slice(&["-Z", "unstable-options", "-C", "rtic"]);

                args.extend_from_slice(&[
                    self.command(),
                    "--example",
                    example,
                    "--target",
                    target.triple(),
                ]);

                if let Some(feature_name) = features {
                    args.extend_from_slice(&["--features", feature_name]);
                }
                if let Some(flag) = mode.to_flag() {
                    args.push(flag);
                }
                if let Some(ExtraArguments::Other(arguments)) = arguments {
                    // Arguments to cargo size must be passed after "--"
                    args.extend_from_slice(&["--"]);
                    for arg in arguments {
                        args.extend_from_slice(&[arg.as_str()]);
                    }
                }
                args
            }
            CargoCommand::CheckInDir { mode, dir: _ } => {
                let mut args = vec!["+nightly"];
                args.push(self.command());

                if let Some(mode) = mode.to_flag() {
                    args.push(mode);
                }

                args
            }
            CargoCommand::BuildInDir { mode, dir: _ } => {
                let mut args = vec!["+nightly", self.command()];

                if let Some(mode) = mode.to_flag() {
                    args.push(mode);
                }

                args
            }
        }
    }

    fn chdir(&self) -> Option<&PathBuf> {
        match self {
            CargoCommand::CheckInDir { dir, .. } | CargoCommand::BuildInDir { dir, .. } => {
                Some(dir)
            }
            _ => None,
        }
    }

    pub fn print_stdout_intermediate(&self) -> bool {
        match self {
            Self::ExampleSize { .. } => true,
            _ => false,
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

pub fn run_command(command: &CargoCommand, stderr_mode: OutputMode) -> anyhow::Result<RunResult> {
    log::info!("👟 {command}");

    let mut process = Command::new(command.executable());

    process
        .args(command.args())
        .stdout(Stdio::piped())
        .stderr(stderr_mode);

    if let Some(dir) = command.chdir() {
        process.current_dir(dir);
    }

    let result = process.output()?;

    let exit_status = result.status;
    let stderr = String::from_utf8(result.stderr).unwrap_or("Not displayable".into());
    let stdout = String::from_utf8(result.stdout).unwrap_or("Not displayable".into());

    if command.print_stdout_intermediate() && exit_status.success() {
        log::info!("\n{}", stdout);
    }

    if !exit_status.success() {
        log::error!("❌ Command failed. Run to completion for the summary.");
    }

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
                log::log!(level, "\n{cmd}\nStdout:\n{stdout}\nStderr:\n{stderr}");
            } else if !stdout.is_empty() {
                log::log!(level, "\n{cmd}\nStdout:\n{}", stdout.trim_end());
            } else if !stderr.is_empty() {
                log::log!(level, "\n{cmd}\nStderr:\n{}", stderr.trim_end());
            }
        }
    };

    successes.for_each(|(cmd, stdout, stderr)| {
        let path = if let Some(dir) = cmd.chdir() {
            let path = dir.as_os_str().to_str().unwrap_or("Not displayable");
            format!(" (in {path}")
        } else {
            format!("")
        };

        if globals.verbose > 0 {
            info!("✅ Success: {cmd}{path}\n    {}", cmd.as_cmd_string());
        } else {
            info!("✅ Success: {cmd}{path}");
        }

        log_stdout_stderr(Level::Debug)((cmd, stdout, stderr));
    });

    errors.clone().for_each(|(cmd, stdout, stderr)| {
        if let Some(dir) = cmd.chdir() {
            let path = dir.as_os_str().to_str().unwrap_or("Not displayable");
            error!("❌ Failed: {cmd} (in {path}) \n    {}", cmd.as_cmd_string());
        } else {
            error!("❌ Failed: {cmd}\n    {}", cmd.as_cmd_string());
        }
        log_stdout_stderr(Level::Error)((cmd, stdout, stderr));
    });

    command_errors
        .clone()
        .for_each(|(cmd, error)| error!("❌ Failed: {cmd}\n    {}\n{error}", cmd.as_cmd_string()));

    let ecount = errors.count() + command_errors.count();
    if ecount != 0 {
        log::error!("{ecount} commands failed.");
        Err(())
    } else {
        info!("🚀🚀🚀 All tasks succeeded 🚀🚀🚀");
        Ok(())
    }
}
