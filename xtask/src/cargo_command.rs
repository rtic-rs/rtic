use crate::{ExtraArguments, Target};
use core::fmt;
use std::path::PathBuf;

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
        deny_warnings: bool,
    },
    ExampleBuild {
        cargoarg: &'a Option<&'a str>,
        example: &'a str,
        target: Option<Target<'a>>,
        features: Option<String>,
        mode: BuildMode,
        dir: Option<PathBuf>,
        deny_warnings: bool,
    },
    ExampleCheck {
        cargoarg: &'a Option<&'a str>,
        example: &'a str,
        target: Option<Target<'a>>,
        features: Option<String>,
        mode: BuildMode,
        deny_warnings: bool,
    },
    Build {
        cargoarg: &'a Option<&'a str>,
        package: Option<String>,
        target: Option<Target<'a>>,
        features: Option<String>,
        mode: BuildMode,
        dir: Option<PathBuf>,
        deny_warnings: bool,
    },
    Check {
        cargoarg: &'a Option<&'a str>,
        package: Option<String>,
        target: Option<Target<'a>>,
        features: Option<String>,
        mode: BuildMode,
        dir: Option<PathBuf>,
        deny_warnings: bool,
    },
    Clippy {
        cargoarg: &'a Option<&'a str>,
        package: Option<String>,
        target: Option<Target<'a>>,
        features: Option<String>,
        deny_warnings: bool,
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
        deny_warnings: bool,
    },
    Test {
        package: Option<String>,
        features: Option<String>,
        test: Option<String>,
        deny_warnings: bool,
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
        deny_warnings: bool,
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
            deny_warnings: bool,
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
                format!("<no explicit target>")
            };

            let mode = if let Some(mode) = mode {
                format!("{mode}")
            } else {
                format!("debug")
            };

            let deny_warnings = if deny_warnings {
                format!("deny warnings, ")
            } else {
                format!("")
            };

            if cargoarg.is_some() && path.is_some() {
                format!("({deny_warnings}{target}, {mode}, {feat}, {carg}, {in_dir})")
            } else if cargoarg.is_some() {
                format!("({deny_warnings}{target}, {mode}, {feat}, {carg})")
            } else if path.is_some() {
                format!("({deny_warnings}{target}, {mode}, {feat}, {in_dir})")
            } else {
                format!("({deny_warnings}{target}, {mode}, {feat})")
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
                    details(false, target, Some(mode), features, cargoarg, dir.as_ref())
                )
            }
            CargoCommand::Qemu {
                cargoarg,
                example,
                target,
                features,
                mode,
                dir,
                deny_warnings,
            } => {
                let warns = *deny_warnings;
                let details = details(warns, target, Some(mode), features, cargoarg, dir.as_ref());
                write!(f, "Run example {example} in QEMU {details}",)
            }
            CargoCommand::ExampleBuild {
                cargoarg,
                example,
                target,
                features,
                mode,
                dir,
                deny_warnings,
            } => {
                let warns = *deny_warnings;
                let details = details(warns, target, Some(mode), features, cargoarg, dir.as_ref());
                write!(f, "Build example {example} {details}",)
            }
            CargoCommand::ExampleCheck {
                cargoarg,
                example,
                target,
                features,
                mode,
                deny_warnings,
            } => write!(
                f,
                "Check example {example} {}",
                details(*deny_warnings, target, Some(mode), features, cargoarg, None)
            ),
            CargoCommand::Build {
                cargoarg,
                package,
                target,
                features,
                mode,
                dir,
                deny_warnings,
            } => {
                let package = p(package);
                let warns = *deny_warnings;
                write!(
                    f,
                    "Build {package} {}",
                    details(warns, target, Some(mode), features, cargoarg, dir.as_ref())
                )
            }

            CargoCommand::Check {
                cargoarg,
                package,
                target,
                features,
                mode,
                dir,
                deny_warnings,
            } => {
                let package = p(package);
                let warns = *deny_warnings;
                write!(
                    f,
                    "Check {package} {}",
                    details(warns, target, Some(mode), features, cargoarg, dir.as_ref())
                )
            }
            CargoCommand::Clippy {
                cargoarg,
                package,
                target,
                features,
                deny_warnings,
            } => {
                let details = details(*deny_warnings, target, None, features, cargoarg, None);
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
                deny_warnings,
            } => {
                let feat = feat(features);
                let carg = carg(cargoarg);
                let arguments = arguments
                    .clone()
                    .map(|a| format!("{a}"))
                    .unwrap_or_else(|| "no extra arguments".into());
                let deny_warnings = if *deny_warnings {
                    format!("deny warnings, ")
                } else {
                    format!("")
                };
                if cargoarg.is_some() {
                    write!(f, "Document ({deny_warnings}{feat}, {carg}, {arguments})")
                } else {
                    write!(f, "Document ({deny_warnings}{feat}, {arguments})")
                }
            }
            CargoCommand::Test {
                package,
                features,
                test,
                deny_warnings,
            } => {
                let p = p(package);
                let test = test
                    .clone()
                    .map(|t| format!("test {t}"))
                    .unwrap_or("all tests".into());
                let deny_warnings = if *deny_warnings {
                    format!("deny warnings, ")
                } else {
                    format!("")
                };
                let feat = feat(features);
                write!(f, "Run {test} in {p} ({deny_warnings}features: {feat})")
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
                deny_warnings,
            } => {
                let warns = *deny_warnings;
                let details = details(warns, target, Some(mode), features, cargoarg, dir.as_ref());
                write!(f, "Compute size of example {example} {details}")
            }
        }
    }
}

impl<'a> CargoCommand<'a> {
    pub fn as_cmd_string(&self) -> String {
        let env = if let Some((key, value)) = self.extra_env() {
            format!("{key}=\"{value}\" ")
        } else {
            format!("")
        };

        let cd = if let Some(Some(chdir)) = self.chdir().map(|p| p.to_str()) {
            format!("cd {chdir} && ")
        } else {
            format!("")
        };

        let executable = self.executable();
        let args = self.args().join(" ");
        format!("{env}{cd}{executable} {args}")
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
                // deny_warnings is exposed through `extra_env`
                deny_warnings: _,
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
                // Target is added by build_args
                target: _,
                // Dir is exposed through `chdir`
                dir: _,
                // deny_warnings is exposed through `extra_env`
                deny_warnings: _,
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
                // deny_warnings is exposed through `extra_env`
                deny_warnings: _,
            } => self.build_args(true, cargoarg, features, Some(mode), p(package)),
            CargoCommand::Clippy {
                cargoarg,
                package,
                features,
                // Target is added by build_args
                target: _,
                deny_warnings,
            } => {
                let deny_warnings = if *deny_warnings {
                    vec!["--", "-D", "warnings"]
                } else {
                    vec![]
                };

                let extra = p(package).chain(deny_warnings);
                self.build_args(true, cargoarg, features, None, extra)
            }
            CargoCommand::Doc {
                cargoarg,
                features,
                arguments,
                // deny_warnings is exposed through `extra_env`
                deny_warnings: _,
            } => {
                let extra = Self::extra_args(arguments.as_ref());
                self.build_args(true, cargoarg, features, None, extra)
            }
            CargoCommand::Test {
                package,
                features,
                test,
                // deny_warnings is exposed through `extra_env`
                deny_warnings: _,
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
                // deny_warnings is exposed through `extra_env`
                deny_warnings: _,
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
                // deny_warnings is exposed through `extra_env`
                deny_warnings: _,
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
                // deny_warnings is exposed through `extra_env`
                deny_warnings: _,
            } => {
                let extra = ["--example", example]
                    .into_iter()
                    .chain(Self::extra_args(arguments.as_ref()));

                self.build_args(true, cargoarg, features, Some(mode), extra)
            }
        }
    }

    /// TODO: integrate this into `args` once `-C` becomes stable.
    pub fn chdir(&self) -> Option<&PathBuf> {
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

    pub fn extra_env(&self) -> Option<(&str, &str)> {
        match self {
            // Clippy is a special case: it sets deny warnings
            // through an argument to rustc.
            CargoCommand::Clippy { .. } => None,
            CargoCommand::Doc { .. } => Some(("RUSTDOCFLAGS", "-D warnings")),

            CargoCommand::Qemu { deny_warnings, .. }
            | CargoCommand::ExampleBuild { deny_warnings, .. }
            | CargoCommand::ExampleSize { deny_warnings, .. } => {
                if *deny_warnings {
                    // NOTE: this also needs the link-arg because .cargo/config.toml
                    // is ignored if you set the RUSTFLAGS env variable.
                    Some(("RUSTFLAGS", "-D warnings -C link-arg=-Tlink.x"))
                } else {
                    None
                }
            }

            CargoCommand::Check { deny_warnings, .. }
            | CargoCommand::ExampleCheck { deny_warnings, .. }
            | CargoCommand::Build { deny_warnings, .. }
            | CargoCommand::Test { deny_warnings, .. } => {
                if *deny_warnings {
                    Some(("RUSTFLAGS", "-D warnings"))
                } else {
                    None
                }
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
