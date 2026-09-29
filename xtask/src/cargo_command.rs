use crate::{
    argument_parsing::{Package, Platform},
    Target,
};
use core::fmt;
use std::path::PathBuf;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuildMode {
    Release,
    Debug,
}

#[derive(Debug, Clone)]
pub enum CargoCommand {
    // For future embedded-ci
    #[allow(dead_code)]
    Run {
        cargoarg: Option<String>,
        platform: Platform,
        example: String,
        target: Option<Target>,
        features: Option<String>,
        mode: BuildMode,
        dir: Option<PathBuf>,
    },
    Qemu {
        cargoarg: Option<String>,
        platform: Platform,
        example: String,
        mode: BuildMode,
        dir: Option<PathBuf>,
        deny_warnings: bool,
    },
    ExampleBuild {
        cargoarg: Option<String>,
        example: String,
        platform: Platform,
        mode: BuildMode,
        dir: Option<PathBuf>,
        deny_warnings: bool,
    },
    ExampleCheck {
        cargoarg: Option<String>,
        platform: Platform,
        example: String,
        mode: BuildMode,
        dir: Option<PathBuf>,
        deny_warnings: bool,
    },
    Build {
        cargoarg: Option<String>,
        package: Option<Package>,
        target: Option<Target>,
        features: Option<String>,
        mode: BuildMode,
        dir: Option<PathBuf>,
        deny_warnings: bool,
    },
    Check {
        cargoarg: Option<String>,
        package: Option<Package>,
        target: Option<Target>,
        features: Option<String>,
        mode: BuildMode,
        dir: Option<PathBuf>,
        deny_warnings: bool,
    },
    Clippy {
        cargoarg: Option<String>,
        package: Option<Package>,
        target: Option<Target>,
        features: Option<String>,
        deny_warnings: bool,
    },
    Format {
        cargoarg: Option<String>,
        manifest: PathBuf,
        check_only: bool,
    },
    Doc {
        cargoarg: Option<String>,
        features: Option<String>,
        arguments: Vec<String>,
        deny_warnings: bool,
    },
    Test {
        package: Option<Package>,
        features: Option<String>,
        test: Option<String>,
        deny_warnings: bool,
        loom: bool,
    },
    Book {
        arguments: Vec<String>,
    },
    ExampleSize {
        cargoarg: Option<String>,
        platform: Platform,
        example: String,
        mode: BuildMode,
        arguments: Vec<String>,
        dir: Option<PathBuf>,
        deny_warnings: bool,
    },
}

impl core::fmt::Display for CargoCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn p(p: Option<Package>) -> String {
            if let Some(package) = p {
                format!("package {}", package.name())
            } else {
                "default packages".to_string()
            }
        }

        fn feat(f: Option<&String>) -> String {
            if let Some(features) = f {
                format!("\"{features}\"")
            } else {
                "no features".to_string()
            }
        }

        fn carg(f: Option<&str>) -> String {
            if let Some(cargoarg) = f {
                cargoarg.to_string()
            } else {
                "no cargo args".to_string()
            }
        }

        fn details(
            deny_warnings: bool,
            target: Option<Target>,
            mode: Option<BuildMode>,
            features: Option<String>,
            cargoarg: Option<&str>,
            path: Option<&PathBuf>,
            // no need to add platform, as it is implicit in the path
        ) -> String {
            let feat = feat(features.as_ref());
            let carg = carg(cargoarg);
            let in_dir = if let Some(path) = path {
                let path = path.to_str().unwrap_or("<can't display>");
                format!("in {path}")
            } else {
                "".to_string()
            };

            let target = if let Some(target) = target {
                format!("{target}")
            } else {
                "<no explicit target>".to_string()
            };

            let mode = if let Some(mode) = mode {
                format!("{mode}")
            } else {
                "debug".to_string()
            };

            let deny_warnings = if deny_warnings {
                "deny warnings, ".to_string()
            } else {
                "".to_string()
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

        match self.clone() {
            CargoCommand::Run {
                cargoarg,
                platform: _,
                example,
                target,
                features,
                mode,
                dir,
            } => {
                write!(
                    f,
                    "Run example {example} {}",
                    details(
                        false,
                        target,
                        Some(mode),
                        features,
                        cargoarg.as_deref(),
                        dir.as_ref()
                    )
                )
            }
            CargoCommand::Qemu {
                cargoarg,
                platform,
                example,
                mode,
                dir,
                deny_warnings,
            } => {
                let details = details(
                    deny_warnings,
                    Some(platform.target()),
                    Some(mode),
                    platform.example_features(),
                    cargoarg.as_deref(),
                    dir.as_ref(),
                );
                write!(f, "Run example {example} in QEMU {details}",)
            }
            CargoCommand::ExampleBuild {
                cargoarg,
                example,
                platform,
                mode,
                dir,
                deny_warnings,
            } => {
                let details = details(
                    deny_warnings,
                    Some(platform.target()),
                    Some(mode),
                    platform.example_features(),
                    cargoarg.as_deref(),
                    dir.as_ref(),
                );
                write!(f, "Build example {example} {details}",)
            }
            CargoCommand::ExampleCheck {
                cargoarg,
                platform,
                example,
                mode,
                dir,
                deny_warnings,
            } => {
                let details = details(
                    deny_warnings,
                    Some(platform.target()),
                    Some(mode),
                    platform.example_features(),
                    cargoarg.as_deref(),
                    dir.as_ref(),
                );
                write!(f, "Check example {example} {details}",)
            }
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
                write!(
                    f,
                    "Build {package} {}",
                    details(
                        deny_warnings,
                        target,
                        Some(mode),
                        features,
                        cargoarg.as_deref(),
                        dir.as_ref()
                    )
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
                let details = details(
                    deny_warnings,
                    target,
                    Some(mode),
                    features,
                    cargoarg.as_deref(),
                    dir.as_ref(),
                );
                write!(f, "Check {package} {details}",)
            }
            CargoCommand::Clippy {
                cargoarg,
                package,
                target,
                features,
                deny_warnings,
            } => {
                let details = details(
                    deny_warnings,
                    target,
                    None,
                    features,
                    cargoarg.as_deref(),
                    None,
                );
                let package = p(package);
                write!(f, "Clippy {package} {details}")
            }
            CargoCommand::Format {
                cargoarg,
                manifest,
                check_only,
            } => {
                let package = manifest.display().to_string();
                let carg = carg(cargoarg.as_deref());

                let carg = if cargoarg.is_some() {
                    format!("(cargo args: {carg})")
                } else {
                    String::new()
                };

                if check_only {
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
                let feat = feat(features.as_ref());
                let carg = carg(cargoarg.as_deref());
                let arguments = if arguments.is_empty() {
                    arguments.join(" ")
                } else {
                    "no extra argument".to_string()
                };

                let deny_warnings = if deny_warnings {
                    "deny warnings, ".to_string()
                } else {
                    String::new()
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
                loom: _,
            } => {
                let p = p(package);
                let test = test
                    .clone()
                    .map(|t| format!("test {t}"))
                    .unwrap_or("all tests".into());

                let details = details(deny_warnings, None, None, features, None, None);
                write!(f, "Run {test} in {p} {details}")
            }
            CargoCommand::Book { arguments: _ } => write!(f, "Build the book"),
            CargoCommand::ExampleSize {
                cargoarg,
                platform,
                example,
                mode,
                arguments: _,
                dir,
                deny_warnings,
            } => {
                let details = details(
                    deny_warnings,
                    Some(platform.target()),
                    Some(mode),
                    platform.example_features(),
                    cargoarg.as_deref(),
                    dir.as_ref(),
                );
                write!(f, "Compute size of example {example} {details}")
            }
        }
    }
}

impl CargoCommand {
    pub fn as_cmd_string(&self) -> String {
        let env = if let Some((key, value)) = self.extra_env() {
            format!("{key}=\"{value}\" ")
        } else {
            String::new()
        };

        let cd = if let Some(Some(chdir)) = self.chdir().map(|p| p.to_str()) {
            format!("cd {chdir} && ")
        } else {
            String::new()
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
    fn build_args<S: Into<String>, T: Iterator<Item = S>>(
        command: &str,
        target: Option<Target>,
        nightly: bool,
        cargoarg: Option<String>,
        features: Option<String>,
        mode: Option<&BuildMode>,
        extra: T,
    ) -> Vec<String> {
        let mut args: Vec<String> = Vec::new();

        if nightly {
            args.push("+nightly".to_owned());
        }

        args.extend(cargoarg);
        args.push(command.to_owned());

        if let Some(target) = target {
            args.push("--target".to_owned());
            args.push(target.triple().to_owned());
        }

        if let Some(features) = features {
            args.push("--features".to_owned());
            args.push(features);
        }

        if let Some(mode) = mode.and_then(|m| m.to_flag()) {
            args.push(mode.to_owned());
        }

        args.extend(extra.map(Into::into));

        args
    }

    /// Turn the ExtraArguments into an interator that contains the separating dashes
    /// and the rest of the arguments.
    ///
    /// NOTE: you _must_ chain this iterator at the _end_ of the extra arguments.
    fn extra_args(args: &[String]) -> impl Iterator<Item = &str> {
        let args = if !args.is_empty() {
            // Extra arguments must be passed after "--"
            ["--"]
                .into_iter()
                .chain(args.iter().map(String::as_str))
                .collect()
        } else {
            vec![]
        };
        args.into_iter()
    }

    pub fn args(&self) -> Vec<String> {
        fn p(package: &Option<Package>) -> impl Iterator<Item = &str> {
            if let Some(package) = package {
                vec!["--package", package.name()].into_iter()
            } else {
                vec![].into_iter()
            }
        }

        match self {
            // For future embedded-ci, for now the same as Qemu
            CargoCommand::Run {
                cargoarg,
                platform: _,
                example,
                features,
                mode,
                // dir is exposed through `chdir`
                dir: _,
                target,
            } => Self::build_args(
                self.command(),
                *target,
                false,
                cargoarg.clone(),
                features.clone(),
                Some(mode),
                ["--example", example].into_iter(),
            ),
            CargoCommand::Qemu {
                cargoarg,
                platform,
                example,
                mode,
                // dir is exposed through `chdir`
                dir: _,
                // deny_warnings is exposed through `extra_env`
                deny_warnings: _,
            } => Self::build_args(
                self.command(),
                Some(platform.target()),
                false,
                cargoarg.clone(),
                platform.example_features(),
                Some(mode),
                ["--example", example].into_iter(),
            ),
            CargoCommand::Build {
                cargoarg,
                package,
                features,
                mode,
                target,
                // Dir is exposed through `chdir`
                dir: _,
                // deny_warnings is exposed through `extra_env`
                deny_warnings: _,
            } => Self::build_args(
                self.command(),
                *target,
                false,
                cargoarg.clone(),
                features.clone(),
                Some(mode),
                p(package),
            ),
            CargoCommand::Check {
                cargoarg,
                package,
                features,
                mode,
                // Dir is exposed through `chdir`
                dir: _,
                target,
                // deny_warnings is exposed through `extra_env`
                deny_warnings: _,
            } => Self::build_args(
                self.command(),
                *target,
                false,
                cargoarg.clone(),
                features.clone(),
                Some(mode),
                p(package),
            ),
            CargoCommand::Clippy {
                cargoarg,
                package,
                features,
                target,
                deny_warnings,
            } => {
                let deny_warnings = if *deny_warnings {
                    vec!["--", "-D", "warnings"]
                } else {
                    vec![]
                };

                let extra = p(package).chain(deny_warnings);
                Self::build_args(
                    self.command(),
                    *target,
                    false,
                    cargoarg.clone(),
                    features.clone(),
                    None,
                    extra,
                )
            }
            CargoCommand::Doc {
                cargoarg,
                features,
                arguments,
                // deny_warnings is exposed through `extra_env`
                deny_warnings: _,
            } => {
                let extra = arguments.iter().map(String::as_str);
                Self::build_args(
                    self.command(),
                    None,
                    false,
                    cargoarg.clone(),
                    features.clone(),
                    None,
                    extra,
                )
            }
            CargoCommand::Test {
                package,
                features,
                test,
                // deny_warnings is exposed through `extra_env`
                deny_warnings: _,
                loom,
            } => {
                let mut extra = if let Some(test) = test {
                    vec!["--test", test]
                } else {
                    vec![]
                };

                let feats = if *loom {
                    extra.push(" --lib");
                    None
                } else {
                    features.as_ref().map(|v| v.to_owned())
                };

                let package = p(package);
                let extra = extra.into_iter().chain(package);
                Self::build_args(self.command(), None, false, None, feats, None, extra)
            }
            CargoCommand::Book { arguments } => {
                let mut args = vec![];

                if !arguments.is_empty() {
                    args.extend(arguments.iter().map(Clone::clone));
                } else {
                    // If no argument given, run mdbook build
                    // with default path to book
                    args.push(self.command().to_owned());
                    args.push("book/en".to_owned());
                }
                args
            }
            CargoCommand::Format {
                cargoarg,
                manifest,
                check_only,
            } => {
                let extra = if *check_only { Some("--check") } else { None };
                let package = [
                    "--manifest-path".to_string(),
                    manifest.display().to_string(),
                    "--all".to_string(),
                ];
                Self::build_args(
                    self.command(),
                    None,
                    false,
                    cargoarg.clone(),
                    None,
                    None,
                    extra.into_iter().map(String::from).chain(package),
                )
            }
            CargoCommand::ExampleBuild {
                cargoarg,
                example,
                platform,
                mode,
                // dir is exposed through `chdir`
                dir: _,
                // deny_warnings is exposed through `extra_env`
                deny_warnings: _,
            } => Self::build_args(
                self.command(),
                Some(platform.target()),
                false,
                cargoarg.clone(),
                platform.example_features(),
                Some(mode),
                ["--example", example].into_iter(),
            ),
            CargoCommand::ExampleCheck {
                cargoarg,
                platform,
                example,
                mode,
                dir: _,
                // deny_warnings is exposed through `extra_env`
                deny_warnings: _,
            } => Self::build_args(
                self.command(),
                Some(platform.target()),
                false,
                cargoarg.clone(),
                platform.example_features(),
                Some(mode),
                ["--example", example].into_iter(),
            ),
            CargoCommand::ExampleSize {
                cargoarg,
                platform,
                example,
                mode,
                arguments,
                // dir is exposed through `chdir`
                dir: _,
                // deny_warnings is exposed through `extra_env`
                deny_warnings: _,
            } => {
                let extra = ["--example", example]
                    .into_iter()
                    .chain(Self::extra_args(arguments));

                Self::build_args(
                    self.command(),
                    Some(platform.target()),
                    false,
                    cargoarg.clone(),
                    platform.example_features(),
                    Some(mode),
                    extra,
                )
            }
        }
    }

    /// TODO: integrate this into `args` once `-C` becomes stable.
    pub fn chdir(&self) -> Option<&PathBuf> {
        match self {
            CargoCommand::Qemu { dir, .. }
            | CargoCommand::ExampleCheck { dir, .. }
            | CargoCommand::ExampleBuild { dir, .. }
            | CargoCommand::ExampleSize { dir, .. }
            | CargoCommand::Build { dir, .. }
            | CargoCommand::Run { dir, .. }
            | CargoCommand::Check { dir, .. } => dir.as_ref(),
            _ => None,
        }
    }

    pub fn extra_env(&self) -> Option<(&str, String)> {
        match self {
            // Clippy is a special case: it sets deny warnings
            // through an argument to rustc.
            CargoCommand::Clippy { .. } => None,
            CargoCommand::Doc { .. } => Some(("RUSTDOCFLAGS", "-D warnings".to_string())),

            CargoCommand::Qemu { deny_warnings, .. }
            | CargoCommand::ExampleBuild { deny_warnings, .. }
            | CargoCommand::ExampleSize { deny_warnings, .. } => {
                if *deny_warnings {
                    Some(("RUSTFLAGS", "-D warnings".to_string()))
                // TODO make this configurable
                } else {
                    None
                }
            }

            CargoCommand::Check { deny_warnings, .. }
            | CargoCommand::ExampleCheck { deny_warnings, .. }
            | CargoCommand::Build { deny_warnings, .. } => {
                if *deny_warnings {
                    Some(("RUSTFLAGS", "-D warnings".to_string()))
                } else {
                    None
                }
            }
            CargoCommand::Test {
                deny_warnings,
                loom,
                ..
            } => {
                let mut combined_flags = vec![""];

                if *deny_warnings {
                    combined_flags.push("-D warnings");
                }
                if *loom {
                    combined_flags.push("--cfg loom");
                }
                if !combined_flags.is_empty() {
                    let rust_flags = combined_flags.join(" ").to_string();
                    Some(("RUSTFLAGS", rust_flags))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn print_stdout_intermediate(&self) -> bool {
        matches!(self, Self::ExampleSize { .. })
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
