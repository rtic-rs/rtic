use crate::{
    cargo_command::CargoCommand, Target, ARMV6M, ARMV7M, ARMV8MBASE, ARMV8MMAIN, RISCV32IMAC,
    RISCV32IMC,
};
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
    pub fn name(&self) -> String {
        let name = match self {
            Package::Rtic => "rtic",
            Package::RticCommon => "rtic-common",
            Package::RticMacros => "rtic-macros",
            Package::RticMonotonics => "rtic-monotonics",
            Package::RticSync => "rtic-sync",
            Package::RticTime => "rtic-time",
        };

        name.to_string()
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
    pub fn features(
        &self,
        target: Target,
        backend: Backends,
        partial: bool,
    ) -> Vec<Option<String>> {
        match self {
            Package::Rtic => vec![Some(target.and_features(backend.to_rtic_feature()))],
            Package::RticMacros => {
                vec![Some(backend.to_rtic_macros_feature().to_string())]
            }
            Package::RticMonotonics => {
                let features = backend.to_rtic_monotonics_features(partial);

                if let Some(features) = features {
                    features
                        .iter()
                        .map(|&s| s.to_string())
                        .map(Some)
                        .chain(std::iter::once(None))
                        .collect()
                } else {
                    vec![None]
                }
            }
            Package::RticSync if matches!(backend, Backends::Thumbv6) || !backend.is_arm() => {
                vec![Some("portable-atomic/critical-section".into())]
            }
            _ => vec![None],
        }
    }
}

pub struct TestMetadata {}

impl TestMetadata {
    pub fn match_package(package: Package, backend: Backends, loom: bool) -> CargoCommand<'static> {
        match package {
            Package::Rtic => {
                let features = Some(backend.to_target().and_features(backend.to_rtic_feature()));
                CargoCommand::Test {
                    package: Some(package.name()),
                    features,
                    test: Some("ui".to_owned()),
                    deny_warnings: true,
                    loom,
                }
            }
            Package::RticMacros => CargoCommand::Test {
                package: Some(package.name()),
                features: Some(backend.to_rtic_macros_feature().to_owned()),
                test: None,
                deny_warnings: true,
                loom,
            },
            Package::RticSync => CargoCommand::Test {
                package: Some(package.name()),
                features: Some("testing".to_owned()),
                test: None,
                deny_warnings: true,
                loom,
            },
            Package::RticCommon => CargoCommand::Test {
                package: Some(package.name()),
                features: Some("testing".to_owned()),
                test: None,
                deny_warnings: true,
                loom,
            },
            Package::RticMonotonics => CargoCommand::Test {
                package: Some(package.name()),
                features: None,
                test: None,
                deny_warnings: true,
                loom,
            },
            Package::RticTime => CargoCommand::Test {
                package: Some(package.name()),
                features: Some("critical-section/std".into()),
                test: None,
                deny_warnings: true,
                loom,
            },
        }
    }
}

#[derive(clap::ValueEnum, Copy, Clone, Default, Debug, PartialEq)]
pub enum Backends {
    Thumbv6,
    #[default]
    Thumbv7,
    Thumbv8Base,
    Thumbv8Main,
    RiscvEsp32C3,
    RiscvEsp32C6,
    Riscv32ImcClint,
    Riscv32ImcMecall,
    Riscv32ImacClint,
    Riscv32ImacMecall,
}

impl Backends {
    #[allow(clippy::wrong_self_convention)]
    pub fn to_target(&self) -> Target<'static> {
        match self {
            Backends::Thumbv6 => ARMV6M,
            Backends::Thumbv7 => ARMV7M,
            Backends::Thumbv8Base => ARMV8MBASE,
            Backends::Thumbv8Main => ARMV8MMAIN,
            Backends::Riscv32ImcClint | Backends::Riscv32ImcMecall | Backends::RiscvEsp32C3 => {
                RISCV32IMC
            }
            Backends::Riscv32ImacClint | Backends::Riscv32ImacMecall | Backends::RiscvEsp32C6 => {
                RISCV32IMAC
            }
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_rtic_feature(&self) -> &'static str {
        match self {
            Backends::Thumbv6 => "thumbv6-backend",
            Backends::Thumbv7 => "thumbv7-backend",
            Backends::Thumbv8Base => "thumbv8base-backend",
            Backends::Thumbv8Main => "thumbv8main-backend",
            Backends::RiscvEsp32C3 => "riscv-esp32c3-backend",
            Backends::RiscvEsp32C6 => "riscv-esp32c6-backend",
            Backends::Riscv32ImcClint | Backends::Riscv32ImacClint => "riscv-clint-backend",
            Backends::Riscv32ImcMecall | Backends::Riscv32ImacMecall => "riscv-mecall-backend",
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_rtic_macros_feature(&self) -> &'static str {
        match self {
            Backends::Thumbv6 | Backends::Thumbv8Base => "cortex-m-source-masking",
            Backends::Thumbv7 | Backends::Thumbv8Main => "cortex-m-basepri",
            Backends::RiscvEsp32C3 => "riscv-esp32c3",
            Backends::RiscvEsp32C6 => "riscv-esp32c6",
            Backends::Riscv32ImcClint | Backends::Riscv32ImacClint => "riscv-clint",
            Backends::Riscv32ImcMecall | Backends::Riscv32ImacMecall => "riscv-mecall",
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_rtic_monotonics_features(&self, partial: bool) -> Option<&[&str]> {
        if self == &Self::RiscvEsp32C3 {
            Some(&["esp32c3-systimer"])
        } else if !self.is_arm() {
            None
        } else if partial {
            Some(&[
                "cortex-m-systick",
                "rp2040",
                "nrf52840",
                "imxrt_gpt1,imxrt-ral/imxrt1062",
                "stm32_tim2,stm32h725ag",
            ])
        } else {
            Some(&[
                "cortex-m-systick",
                "cortex-m-systick,systick-64bit",
                "rp2040",
                "nrf52805",
                "nrf52810",
                "nrf52811",
                "nrf52832",
                "nrf52833",
                "nrf52840",
                "nrf5340-app",
                "nrf5340-net",
                "nrf9160",
                "imxrt_gpt1,imxrt_gpt2,imxrt-ral/imxrt1062",
                "stm32_tim2,stm32_tim3,stm32_tim4,stm32_tim5,stm32_tim15,stm32h725ag",
            ])
        }
    }

    pub fn is_arm(&self) -> bool {
        matches!(
            self,
            Self::Thumbv6 | Self::Thumbv7 | Self::Thumbv8Base | Self::Thumbv8Main
        )
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub enum BuildOrCheck {
    #[default]
    Check,
    Build,
}

#[derive(clap::ValueEnum, Copy, Clone, Default, Debug)]
pub enum Platforms {
    Esp32C3,
    Esp32C6,
    Hifive1,
    #[default]
    Lm3s6965,
    Nrf52840,
    Rp2040,
    Stm32f3,
    Stm32f411,
    Teensy4,
}

impl Platforms {
    pub fn name(&self) -> String {
        let name = match self {
            Platforms::Esp32C3 => "esp32c3",
            Platforms::Esp32C6 => "esp32c6",
            Platforms::Hifive1 => "hifive1",
            Platforms::Lm3s6965 => "lm3s6965",
            Platforms::Nrf52840 => "nrf52840",
            Platforms::Rp2040 => "rp2040",
            Platforms::Stm32f3 => "stm32f3",
            Platforms::Stm32f411 => "stm32f411",
            Platforms::Teensy4 => "teensy4",
        };
        name.to_string()
    }

    /// Rust flags needed for the platform when building
    pub fn rust_flags(&self) -> Vec<String> {
        let c = "-C".to_string();
        match self {
            Platforms::Esp32C3 | Platforms::Esp32C6 => vec![c, "link-arg=-Tlinkall.x".to_string()],
            Platforms::Hifive1 => vec![c, "link-arg=-Thifive1-link.x".to_string()],
            Platforms::Lm3s6965 => vec![c, "link-arg=-Tlink.x".to_string()],
            Platforms::Nrf52840 => vec![
                c.clone(),
                "linker=flip-link".to_string(),
                c.clone(),
                "link-arg=-Tlink.x".to_string(),
                c.clone(),
                "link-arg=-Tdefmt.x".to_string(),
                c,
                "link-arg=--nmagic".to_string(),
            ],
            Platforms::Rp2040 => vec![
                c.clone(),
                "link-arg=--nmagic".to_string(),
                c,
                "link-arg=-Tlink.x".to_string(),
            ],
            Platforms::Stm32f3 => vec![
                c.clone(),
                "link-arg=--nmagic".to_string(),
                c,
                "link-arg=-Tlink.x".to_string(),
            ],
            Platforms::Stm32f411 => vec![
                c.clone(),
                "link-arg=-Tlink.x".to_string(),
                c,
                "link-arg=-Tdefmt.x".to_string(),
            ],
            Platforms::Teensy4 => vec![c, "link-arg=-Tt4link.x".to_string()],
        }
    }

    /// Get the default backend for the platform
    pub fn default_backend(&self) -> Backends {
        match self {
            Platforms::Esp32C3 => Backends::RiscvEsp32C3,
            Platforms::Esp32C6 => Backends::RiscvEsp32C6,
            Platforms::Hifive1 => Backends::Riscv32ImcClint,
            Platforms::Lm3s6965 => Backends::Thumbv7,
            Platforms::Nrf52840 => unimplemented!(),
            Platforms::Rp2040 => unimplemented!(),
            Platforms::Stm32f3 => unimplemented!(),
            Platforms::Stm32f411 => unimplemented!(),
            Platforms::Teensy4 => unimplemented!(),
        }
    }

    /// Get the features needed given the selected platform and backend.
    /// If the backend is not supported for the platform, return Err.
    /// If the backend is supported, but no special features are needed, return Ok(None).
    pub fn features(&self, backend: &Backends) -> Result<Option<&'static str>, ()> {
        match self {
            Platforms::Esp32C3 => match backend.to_target() {
                RISCV32IMC => Ok(None),
                _ => Err(()),
            },
            Platforms::Esp32C6 => match backend.to_target() {
                RISCV32IMAC => Ok(None),
                _ => Err(()),
            },
            Platforms::Hifive1 => match backend.to_target() {
                RISCV32IMC => Ok(None),
                _ => Err(()),
            },
            Platforms::Lm3s6965 => match backend.to_target() {
                ARMV6M => Ok(Some("thumbv6-backend")),
                ARMV7M => Ok(Some("thumbv7-backend")),
                ARMV8MBASE => Ok(Some("thumbv8base-backend")),
                ARMV8MMAIN => Ok(Some("thumbv8main-backend")),
                _ => Err(()),
            },
            Platforms::Nrf52840 => unimplemented!(),
            Platforms::Rp2040 => unimplemented!(),
            Platforms::Stm32f3 => unimplemented!(),
            Platforms::Stm32f411 => unimplemented!(),
            Platforms::Teensy4 => unimplemented!(),
        }
    }
}

#[derive(Parser, Clone)]
pub struct Globals {
    /// Error out on warnings
    #[arg(short = 'D', long)]
    pub deny_warnings: bool,

    /// For which platform to build.
    ///
    /// If omitted, the default platform (i.e., lm3s6965) is used.
    ///
    /// Example: `cargo xtask --platform lm3s6965`
    #[arg(value_enum, short, default_value = "lm3s6965", long, global = true)]
    pub platform: Option<Platforms>,

    /// For which backend to build.
    ///
    /// If omitted, the default backend for the selected platform is used
    /// (check [`Platforms::default_backend`]).
    #[arg(value_enum, short, long, global = true)]
    pub backend: Option<Backends>,

    /// List of comma separated examples to include, all others are excluded
    ///
    /// If omitted all examples are included
    ///
    /// Example: `cargo xtask --example complex,spawn,init`
    /// would include complex, spawn and init
    #[arg(short, long, group = "example_group", global = true)]
    pub example: Option<String>,

    /// List of comma separated examples to exclude, all others are included
    ///
    /// If omitted all examples are included
    ///
    /// Example: `cargo xtask --excludeexample complex,spawn,init`
    /// would exclude complex, spawn and init
    #[arg(long, group = "example_group", global = true)]
    pub exampleexclude: Option<String>,

    /// Enable more verbose output, repeat up to `-vvv` for even more
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Enable `stderr` inheritance on child processes.
    ///
    /// If this flag is enabled, the output of `stderr` produced by child
    /// processes is printed directly to `stderr`. This will cause a lot of
    /// clutter, but can make debugging long-running processes a lot easier.
    #[arg(short, long, global = true)]
    pub stderr_inherited: bool,

    /// Don't build/check/test all feature combinations that are available, only
    /// a necessary subset.
    #[arg(long, global = true)]
    pub partial: bool,
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
    /// Run everything CI would
    #[clap(alias = "ci")]
    AllCi(CiOpt),

    /// Format code
    #[clap(alias = "fmt")]
    Format(FormatOpt),

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
    Size(ArgsAndOverwrite),

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
    Test(TestOpt),

    /// Build books with mdbook
    Book(Arg),
}

#[derive(Args, Debug, Clone, Default)]
pub struct CiOpt {
    #[clap(short, long)]
    pub failearly: bool,
}

#[derive(Args, Debug, Clone, Default)]
pub struct FormatOpt {
    #[clap(flatten)]
    pub package: PackageOpt,
    /// Check-only, do not apply formatting fixes.
    #[clap(short, long)]
    pub check: bool,
}

#[derive(Args, Debug, Clone, Default)]
pub struct TestOpt {
    #[clap(flatten)]
    pub package: PackageOpt,
    /// Should tests be loom tests
    #[clap(short, long)]
    pub loom: bool,
}

#[derive(Args, Debug, Clone, Copy, Default)]
/// Restrict to package, or run on whole workspace
pub struct PackageOpt {
    /// For which package/workspace member to operate
    ///
    /// If omitted, work on all
    pub package: Option<Package>,
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
pub struct ArgsAndOverwrite {
    /// If expected output is missing or mismatching, recreate the file
    ///
    /// This overwrites only missing or mismatching
    #[arg(long)]
    pub overwrite_expected: bool,

    /// Options to pass to `cargo <subcommand>`
    #[command(subcommand)]
    pub arguments: Option<ExtraArguments>,
}

#[derive(Debug, Parser, Clone)]
pub struct Arg {
    /// Options to pass to `cargo <subcommand>`
    #[command(subcommand)]
    pub arguments: Option<ExtraArguments>,
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub enum ExtraArguments {
    /// All remaining flags and options
    #[command(external_subcommand)]
    Other(Vec<String>),
}

impl core::fmt::Display for ExtraArguments {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExtraArguments::Other(args) => {
                write!(f, "{}", args.join(" "))
            }
        }
    }
}
