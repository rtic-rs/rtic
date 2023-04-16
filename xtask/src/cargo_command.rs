use log::{error, info, Level};

use crate::{
    argument_parsing::Globals, xtasks::FinalRunResult, ExtraArguments, RunResult, Target,
    TestRunError,
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
        target: Option<Target<'a>>,
        features: Option<String>,
        mode: BuildMode,
        dir: Option<PathBuf>,
    },
    Qemu {
        cargoarg: &'a Option<&'a str>,
        example: &'a str,
        target: Option<Target<'a>>,
        features: Option<String>,
        mode: BuildMode,
        dir: Option<PathBuf>,
    },
    ExampleBuild {
        cargoarg: &'a Option<&'a str>,
        example: &'a str,
        target: Option<Target<'a>>,
        features: Option<String>,
        mode: BuildMode,
        dir: Option<PathBuf>,
    },
    ExampleCheck {
        cargoarg: &'a Option<&'a str>,
        example: &'a str,
        target: Option<Target<'a>>,
        features: Option<String>,
        mode: BuildMode,
    },
    Build {
        cargoarg: &'a Option<&'a str>,
        package: Option<String>,
        target: Option<Target<'a>>,
        features: Option<String>,
        mode: BuildMode,
        dir: Option<PathBuf>,
    },
    Check {
        cargoarg: &'a Option<&'a str>,
        package: Option<String>,
        target: Option<Target<'a>>,
        features: Option<String>,
        mode: BuildMode,
        dir: Option<PathBuf>,
    },
    Clippy {
        cargoarg: &'a Option<&'a str>,
        package: Option<String>,
        target: Option<Target<'a>>,
        features: Option<String>,
    },
    Format {
        cargoarg: &'a Option<&'a str>,
        package: Option<String>,
        check_only: bool,
    },
    Doc {
        cargoarg: &'a Option<&'a str>,
        features: Option<String>,
        arguments: Option<ExtraArguments>,
    },
    Test {
        package: Option<String>,
        features: Option<String>,
        test: Option<String>,
    },
    Book {
        arguments: Option<ExtraArguments>,
    },
    ExampleSize {
        cargoarg: &'a Option<&'a str>,
        example: &'a str,
        target: Option<Target<'a>>,
        features: Option<String>,
        mode: BuildMode,
        arguments: Option<ExtraArguments>,
        dir: Option<PathBuf>,
    },
}

impl core::fmt::Display for CargoCommand<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn p(p: &Option<String>) -> String {
            if let Some(package) = p {
                format!("package {package}")
            } else {
                format!("default package")
            }
        }

        fn feat(f: &Option<String>) -> String {
            if let Some(features) = f {
                format!("\"{features}\"")
            } else {
                format!("no features")
            }
        }

        fn carg(f: &&Option<&str>) -> String {
            if let Some(cargoarg) = f {
                format!("{cargoarg}")
            } else {
                format!("no cargo args")
            }
        }

        fn details(
            target: &Option<Target>,
            mode: Option<&BuildMode>,
            features: &Option<String>,
            cargoarg: &&Option<&str>,
            path: Option<&PathBuf>,
        ) -> String {
            let feat = feat(features);
            let carg = carg(cargoarg);
            let in_dir = if let Some(path) = path {
                let path = path.to_str().unwrap_or("<can't display>");
                format!("in {path}")
            } else {
                format!("")
            };

            let target = if let Some(target) = target {
                format!("{target}")
            } else {
                format!("<host target>")
            };

            let mode = if let Some(mode) = mode {
                format!("{mode}")
            } else {
                format!("debug")
            };

            if cargoarg.is_some() && path.is_some() {
                format!("({target}, {mode}, {feat}, {carg}, {in_dir})")
            } else if cargoarg.is_some() {
                format!("({target}, {mode}, {feat}, {carg})")
            } else if path.is_some() {
                format!("({target}, {mode}, {feat}, {in_dir})")
            } else {
                format!("({target}, {mode}, {feat})")
            }
        }

        match self {
            CargoCommand::Run {
                cargoarg,
                example,
                target,
                features,
                mode,
                dir,
            } => {
                write!(
                    f,
                    "Run example {example} {}",
                    details(target, Some(mode), features, cargoarg, dir.as_ref())
                )
            }
            CargoCommand::Qemu {
                cargoarg,
                example,
                target,
                features,
                mode,
                dir,
            } => {
                let details = details(target, Some(mode), features, cargoarg, dir.as_ref());
                write!(f, "Run example {example} in QEMU {details}",)
            }
            CargoCommand::ExampleBuild {
                cargoarg,
                example,
                target,
                features,
                mode,
                dir,
            } => {
                let details = details(target, Some(mode), features, cargoarg, dir.as_ref());
                write!(f, "Build example {example} {details}",)
            }
            CargoCommand::ExampleCheck {
                cargoarg,
                example,
                target,
                features,
                mode,
            } => write!(
                f,
                "Check example {example} {}",
                details(target, Some(mode), features, cargoarg, None)
            ),
            CargoCommand::Build {
                cargoarg,
                package,
                target,
                features,
                mode,
                dir,
            } => {
                let package = p(package);
                write!(
                    f,
                    "Build {package} {}",
                    details(target, Some(mode), features, cargoarg, dir.as_ref())
                )
            }

            CargoCommand::Check {
                cargoarg,
                package,
                target,
                features,
                mode,
                dir,
            } => {
                let package = p(package);
                write!(
                    f,
                    "Check {package} {}",
                    details(target, Some(mode), features, cargoarg, dir.as_ref())
                )
            }
            CargoCommand::Clippy {
                cargoarg,
                package,
                target,
                features,
            } => {
                let details = details(target, None, features, cargoarg, None);
                let package = p(package);
                write!(f, "Clippy {package} {details}")
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
                dir,
            } => {
                let details = details(target, Some(mode), features, cargoarg, dir.as_ref());
                write!(f, "Compute size of example {example} {details}")
            }
        }
    }
}

impl<'a> CargoCommand<'a> {
    pub fn as_cmd_string(&self) -> String {
        let cd = if let Some(Some(chdir)) = self.chdir().map(|p| p.to_str()) {
            format!("cd {chdir} && ")
        } else {
            format!("")
        };

        let executable = self.executable();
        let args = self.args().join(" ");
        format!("{cd}{executable} {args}")
    }

    fn command(&self) -> &'static str {
        match self {
            CargoCommand::Run { .. } | CargoCommand::Qemu { .. } => "run",
            CargoCommand::ExampleCheck { .. } | CargoCommand::Check { .. } => "check",
            CargoCommand::ExampleBuild { .. } | CargoCommand::Build { .. } => "build",
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
            | CargoCommand::Doc { .. } => "cargo",
            CargoCommand::Book { .. } => "mdbook",
        }
    }

    /// Build args using common arguments for all commands, and the
    /// specific information provided
    fn build_args<'i, T: Iterator<Item = &'i str>>(
        &'i self,
        nightly: bool,
        cargoarg: &'i Option<&'i str>,
        features: &'i Option<String>,
        mode: Option<&'i BuildMode>,
        extra: T,
    ) -> Vec<&str> {
        let mut args: Vec<&str> = Vec::new();

        if nightly {
            args.push("+nightly");
        }

        if let Some(cargoarg) = cargoarg.as_deref() {
            args.push(cargoarg);
        }

        args.push(self.command());

        if let Some(target) = self.target() {
            args.extend_from_slice(&["--target", target.triple()])
        }

        if let Some(features) = features.as_ref() {
            args.extend_from_slice(&["--features", features]);
        }

        if let Some(mode) = mode.map(|m| m.to_flag()).flatten() {
            args.push(mode);
        }

        args.extend(extra);

        args
    }

    /// Turn the ExtraArguments into an interator that contains the separating dashes
    /// and the rest of the arguments.
    ///
    /// NOTE: you _must_ chain this iterator at the _end_ of the extra arguments.
    fn extra_args(args: Option<&ExtraArguments>) -> impl Iterator<Item = &str> {
        #[allow(irrefutable_let_patterns)]
        let args = if let Some(ExtraArguments::Other(arguments)) = args {
            // Extra arguments must be passed after "--"
            ["--"]
                .into_iter()
                .chain(arguments.iter().map(String::as_str))
                .collect()
        } else {
            vec![]
        };
        args.into_iter()
    }

    pub fn args(&self) -> Vec<&str> {
        fn p(package: &Option<String>) -> impl Iterator<Item = &str> {
            if let Some(package) = package {
                vec!["--package", &package].into_iter()
            } else {
                vec![].into_iter()
            }
        }

        match self {
            // For future embedded-ci, for now the same as Qemu
            CargoCommand::Run {
                cargoarg,
                example,
                features,
                mode,
                // dir is exposed through `chdir`
                dir: _,
                // Target is added by build_args
                target: _,
            } => self.build_args(
                true,
                cargoarg,
                features,
                Some(mode),
                ["--example", example].into_iter(),
            ),
            CargoCommand::Qemu {
                cargoarg,
                example,
                features,
                mode,
                // dir is exposed through `chdir`
                dir: _,
                // Target is added by build_args
                target: _,
            } => self.build_args(
                true,
                cargoarg,
                features,
                Some(mode),
                ["--example", example].into_iter(),
            ),
            CargoCommand::Build {
                cargoarg,
                package,
                features,
                mode,
                // Dir is exposed through `chdir`
                dir: _,
                // Target is added by build_args
                target: _,
            } => self.build_args(true, cargoarg, features, Some(mode), p(package)),
            CargoCommand::Check {
                cargoarg,
                package,
                features,
                mode,
                // Dir is exposed through `chdir`
                dir: _,
                // Target is added by build_args
                target: _,
            } => self.build_args(true, cargoarg, features, Some(mode), p(package)),
            CargoCommand::Clippy {
                cargoarg,
                package,
                features,
                // Target is added by build_args
                target: _,
            } => self.build_args(true, cargoarg, features, None, p(package)),
            CargoCommand::Doc {
                cargoarg,
                features,
                arguments,
            } => {
                let extra = Self::extra_args(arguments.as_ref());
                self.build_args(true, cargoarg, features, None, extra)
            }
            CargoCommand::Test {
                package,
                features,
                test,
            } => {
                let extra = if let Some(test) = test {
                    vec!["--test", test]
                } else {
                    vec![]
                };
                let package = p(package);
                let extra = extra.into_iter().chain(package);
                self.build_args(true, &None, features, None, extra)
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
                let extra = if *check_only { Some("--check") } else { None };
                let package = p(package);
                self.build_args(
                    true,
                    cargoarg,
                    &None,
                    None,
                    extra.into_iter().chain(package),
                )
            }
            CargoCommand::ExampleBuild {
                cargoarg,
                example,
                features,
                mode,
                // dir is exposed through `chdir`
                dir: _,
                // Target is added by build_args
                target: _,
            } => self.build_args(
                true,
                cargoarg,
                features,
                Some(mode),
                ["--example", example].into_iter(),
            ),
            CargoCommand::ExampleCheck {
                cargoarg,
                example,
                features,
                mode,
                // Target is added by build_args
                target: _,
            } => self.build_args(
                true,
                cargoarg,
                features,
                Some(mode),
                ["--example", example].into_iter(),
            ),
            CargoCommand::ExampleSize {
                cargoarg,
                example,
                features,
                mode,
                arguments,
                // Target is added by build_args
                target: _,
                // dir is exposed through `chdir`
                dir: _,
            } => {
                let extra = ["--example", example]
                    .into_iter()
                    .chain(Self::extra_args(arguments.as_ref()));

                self.build_args(true, cargoarg, features, Some(mode), extra)
            }
        }
    }

    /// TODO: integrate this into `args` once `-C` becomes stable.
    fn chdir(&self) -> Option<&PathBuf> {
        match self {
            CargoCommand::Qemu { dir, .. }
            | CargoCommand::ExampleBuild { dir, .. }
            | CargoCommand::ExampleSize { dir, .. }
            | CargoCommand::Build { dir, .. }
            | CargoCommand::Run { dir, .. }
            | CargoCommand::Check { dir, .. } => dir.as_ref(),
            _ => None,
        }
    }

    fn target(&self) -> Option<&Target> {
        match self {
            CargoCommand::Run { target, .. }
            | CargoCommand::Qemu { target, .. }
            | CargoCommand::ExampleBuild { target, .. }
            | CargoCommand::ExampleCheck { target, .. }
            | CargoCommand::Build { target, .. }
            | CargoCommand::Check { target, .. }
            | CargoCommand::Clippy { target, .. }
            | CargoCommand::ExampleSize { target, .. } => target.as_ref(),
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
        process.current_dir(dir.canonicalize()?);
    }

    let result = process.output()?;

    let exit_status = result.status;
    let stderr = String::from_utf8(result.stderr).unwrap_or("Not displayable".into());
    let stdout = String::from_utf8(result.stdout).unwrap_or("Not displayable".into());

    if command.print_stdout_intermediate() && exit_status.success() {
        log::info!("\n{}", stdout);
    }

    if exit_status.success() {
        log::info!("✅ Success.")
    } else {
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
        if globals.verbose > 0 {
            info!("✅ Success: {cmd}\n    {}", cmd.as_cmd_string());
        } else {
            info!("✅ Success: {cmd}");
        }

        log_stdout_stderr(Level::Debug)((cmd, stdout, stderr));
    });

    errors.clone().for_each(|(cmd, stdout, stderr)| {
        error!("❌ Failed: {cmd}\n    {}", cmd.as_cmd_string());
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
