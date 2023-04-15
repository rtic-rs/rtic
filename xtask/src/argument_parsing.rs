use crate::{command::CargoCommand, Target, ARMV6M, ARMV7M, ARMV8MBASE, ARMV8MMAIN};
use clap::{Args, Parser, Subcommand};
use core::fmt;

#[derive(clap::ValueEnum, Copy, Clone, Debug)]
pub enum Package {
    Rtic,
    RticCommon,
    RticMacros,
    RticMonotonics,
    RticSync,
    RticTime,
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Package {
    pub fn name(&self) -> &str {
        match self {
            Package::Rtic => "rtic",
            Package::RticCommon => "rtic-common",
            Package::RticMacros => "rtic-macros",
            Package::RticMonotonics => "rtic-monotonics",
            Package::RticSync => "rtic-sync",
            Package::RticTime => "rtic-time",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::Rtic,
            Self::RticCommon,
            Self::RticMacros,
            Self::RticMonotonics,
            Self::RticSync,
            Self::RticTime,
        ]
    }

    /// Get the features needed given the selected package
    ///
    /// Without package specified the features for RTIC are required
    /// With only a single package which is not RTIC, no special
    /// features are needed
    pub fn extract_features(&self, target: Target, backend: Backends) -> Option<String> {
        match self {
            Package::Rtic => Some(target.and_features(backend.to_rtic_feature())),
            Package::RticMacros => Some(backend.to_rtic_macros_feature().to_owned()),
            _ => None,
        }
    }
}

pub struct TestMetadata {}

impl TestMetadata {
    pub fn match_package(package: Package, backend: Backends) -> CargoCommand<'static> {
        match package {
            Package::Rtic => {
                let features = format!(
                    "{},{}",
                    backend.to_rtic_feature(),
                    backend.to_rtic_uitest_feature()
                );
                let features = Some(backend.to_target().and_features(&features));
                CargoCommand::Test {
                    package: Some(package),
                    features,
                    test: Some("ui".to_owned()),
                }
            }
            Package::RticMacros => CargoCommand::Test {
                package: Some(package),
                features: Some(backend.to_rtic_macros_feature().to_owned()),
                test: None,
            },
            Package::RticSync => CargoCommand::Test {
                package: Some(package),
                features: Some("testing".to_owned()),
                test: None,
            },
            Package::RticCommon => CargoCommand::Test {
                package: Some(package),
                features: Some("testing".to_owned()),
                test: None,
            },
            Package::RticMonotonics => CargoCommand::Test {
                package: Some(package),
                features: None,
                test: None,
            },
            Package::RticTime => CargoCommand::Test {
                package: Some(package),
                features: Some("critical-section/std".into()),
                test: None,
            },
        }
    }
}

#[derive(clap::ValueEnum, Copy, Clone, Default, Debug)]
pub enum Backends {
    Thumbv6,
    #[default]
    Thumbv7,
    Thumbv8Base,
    Thumbv8Main,
}

impl Backends {
    #[allow(clippy::wrong_self_convention)]
    pub fn to_target(&self) -> Target {
        match self {
            Backends::Thumbv6 => ARMV6M,
            Backends::Thumbv7 => ARMV7M,
            Backends::Thumbv8Base => ARMV8MBASE,
            Backends::Thumbv8Main => ARMV8MMAIN,
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_rtic_feature(&self) -> &str {
        match self {
            Backends::Thumbv6 => "thumbv6-backend",
            Backends::Thumbv7 => "thumbv7-backend",
            Backends::Thumbv8Base => "thumbv8base-backend",
            Backends::Thumbv8Main => "thumbv8main-backend",
        }
    }
    #[allow(clippy::wrong_self_convention)]
    pub fn to_rtic_macros_feature(&self) -> &str {
        match self {
            Backends::Thumbv6 | Backends::Thumbv8Base => "cortex-m-source-masking",
            Backends::Thumbv7 | Backends::Thumbv8Main => "cortex-m-basepri",
        }
    }
    #[allow(clippy::wrong_self_convention)]
    pub fn to_rtic_uitest_feature(&self) -> &str {
        match self {
            Backends::Thumbv6 | Backends::Thumbv8Base => "rtic-uitestv6",
            Backends::Thumbv7 | Backends::Thumbv8Main => "rtic-uitestv7",
        }
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub enum BuildOrCheck {
    #[default]
    Check,
    Build,
}

#[derive(Parser, Clone)]
pub struct Globals {
    /// For which backend to build (defaults to thumbv7)
    #[arg(value_enum, short, long)]
    pub backend: Option<Backends>,

    /// List of comma separated examples to include, all others are excluded
    ///
    /// If omitted all examples are included
    ///
    /// Example: `cargo xtask --example complex,spawn,init`
    /// would include complex, spawn and init
    #[arg(short, long, group = "example_group")]
    pub example: Option<String>,

    /// List of comma separated examples to exclude, all others are included
    ///
    /// If omitted all examples are included
    ///
    /// Example: `cargo xtask --excludeexample complex,spawn,init`
    /// would exclude complex, spawn and init
    #[arg(long, group = "example_group")]
    pub exampleexclude: Option<String>,

    /// Enable more verbose output, repeat up to `-vvv` for even more
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Enable `stderr` inheritance on child processes.
    ///
    /// If this flag is enabled, the output of `stderr` produced by child
    /// processes is printed directly to `stderr`. This will cause a lot of
    /// clutter, but can make debugging long-running processes a lot easier.
    #[arg(short, long)]
    pub stderr_inherited: bool,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
/// RTIC xtask powered testing toolbox
pub struct Cli {
    #[clap(flatten)]
    pub globals: Globals,

    /// Subcommand selecting operation
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    /// Check formatting
    FormatCheck(PackageOpt),

    /// Format code
    Format(PackageOpt),

    /// Run clippy
    Clippy(PackageOpt),

    /// Check all packages
    Check(PackageOpt),

    /// Build all packages
    Build(PackageOpt),

    /// Check all examples
    ExampleCheck,

    /// Build all examples
    ExampleBuild,

    /// Run `cargo size` on selected or all examples
    ///
    /// To pass options to `cargo size`, add `--` and then the following
    /// arguments will be passed on
    ///
    /// Example: `cargo xtask size -- -A`
    Size(Arg),

    /// Run examples in QEMU and compare against expected output
    ///
    /// Example runtime output is matched against `rtic/ci/expected/`
    ///
    /// Requires that an ARM target is selected
    Qemu(QemuAndRun),

    /// Run examples through embedded-ci and compare against expected output
    ///
    /// unimplemented!() For now TODO, equal to Qemu
    ///
    /// Example runtime output is matched against `rtic/ci/expected/`
    ///
    /// Requires that an ARM target is selected
    Run(QemuAndRun),

    /// Build docs
    ///
    /// To pass options to `cargo doc`, add `--` and then the following
    /// arguments will be passed on
    ///
    /// Example: `cargo xtask doc -- --open`
    Doc(Arg),

    /// Run tests
    Test(PackageOpt),

    /// Build books with mdbook
    Book(Arg),
}

#[derive(Args, Debug, Clone)]
/// Restrict to package, or run on whole workspace
pub struct PackageOpt {
    /// For which package/workspace member to operate
    ///
    /// If omitted, work on all
    package: Option<Package>,
}

impl PackageOpt {
    #[cfg(not(feature = "rayon"))]
    pub fn packages(&self) -> impl Iterator<Item = Package> {
        self.package
            .map(|p| vec![p])
            .unwrap_or(Package::all())
            .into_iter()
    }

    #[cfg(feature = "rayon")]
    pub fn packages(&self) -> impl rayon::prelude::ParallelIterator<Item = Package> {
        use rayon::prelude::*;
        self.package
            .map(|p| vec![p])
            .unwrap_or(Package::all())
            .into_par_iter()
    }
}

#[derive(Args, Debug, Clone)]
pub struct QemuAndRun {
    /// If expected output is missing or mismatching, recreate the file
    ///
    /// This overwrites only missing or mismatching
    #[arg(long)]
    pub overwrite_expected: bool,
}

#[derive(Debug, Parser, Clone)]
pub struct Arg {
    /// Options to pass to `cargo size`
    #[command(subcommand)]
    pub arguments: Option<ExtraArguments>,
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub enum ExtraArguments {
    /// All remaining flags and options
    #[command(external_subcommand)]
    Other(Vec<String>),
}
