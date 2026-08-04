use crate::{
    cargo_command::CargoCommand, Target, ARMV6M, ARMV7M, ARMV8MBASE, ARMV8MMAIN, RISCV32IMAC,
    RISCV32IMC,
};
use clap::{Args, Parser, Subcommand, ValueEnum};
use core::fmt;

#[derive(Parser, Copy, Clone, Debug)]
pub enum Package {
    Rtic {
        /// The backend to compile RTIC for.
        #[clap(long, short)]
        backend: RticBackend,
    },
    RticCommon,
    RticMacros {
        /// The backend to compile RTIC macros for.
        #[clap(long, short)]
        backend: RticMacrosBackend,
    },
    RticMonotonics {
        #[clap(long, short)]
        backend: RticMonotonicsBackend,
    },
    RticSync,
    RticTime,
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Package {
    pub fn name(&self) -> &'static str {
        match self {
            Package::Rtic { .. } => "rtic",
            Package::RticCommon => "rtic-common",
            Package::RticMacros { .. } => "rtic-macros",
            Package::RticMonotonics { .. } => "rtic-monotonics",
            Package::RticSync => "rtic-sync",
            Package::RticTime => "rtic-time",
        }
    }

    /// Get the target required for this package (returns `None` if the
    /// package is to be built for the host target).
    pub fn target(&self) -> Option<Target> {
        match self {
            Package::Rtic { backend } => Some(backend.target()),
            Package::RticMacros { backend: _ } => None,
            Package::RticMonotonics { backend } => Some(backend.target()),
            Package::RticCommon => None,
            Package::RticSync => None,
            Package::RticTime => None,
        }
    }

    pub fn test_command(&self, loom: bool) -> CargoCommand {
        match self {
            Package::Rtic { backend } => {
                let features = Some(backend.rtic_feature().to_owned());
                CargoCommand::Test {
                    package: Some(*self),
                    features,
                    test: Some("ui".to_owned()),
                    deny_warnings: true,
                    loom,
                }
            }
            Package::RticMacros { backend } => CargoCommand::Test {
                package: Some(*self),
                features: Some(backend.feature().to_owned()),
                test: None,
                deny_warnings: true,
                loom,
            },
            Package::RticSync => CargoCommand::Test {
                package: Some(*self),
                features: None,
                test: None,
                deny_warnings: true,
                loom,
            },
            Package::RticCommon => CargoCommand::Test {
                package: Some(*self),
                features: None,
                test: None,
                deny_warnings: true,
                loom,
            },
            Package::RticMonotonics { backend: _ } => CargoCommand::Test {
                package: Some(*self),
                features: None,
                test: None,
                deny_warnings: true,
                loom,
            },
            Package::RticTime => CargoCommand::Test {
                package: Some(*self),
                features: Some("critical-section/std".into()),
                test: None,
                deny_warnings: true,
                loom,
            },
        }
    }

    /// Get the features needed to compile this backend in a standalone
    /// setup.
    pub fn features(&self, partial: bool) -> Option<String> {
        match self {
            Package::Rtic { backend } => {
                let backend_feature = backend.rtic_feature();
                let cas_feature = self.target().and_then(|v| v.cas_feature());

                Some(if let Some(cas_feature) = cas_feature {
                    format!("{cas_feature},{backend_feature}")
                } else {
                    backend_feature.to_string()
                })
            }
            Package::RticMacros { backend } => Some(backend.feature().to_string()),
            Package::RticMonotonics { backend } => Some(backend.features(partial).join(",")),
            _ => None,
        }
    }

    /// All packages.
    pub fn all() -> impl Iterator<Item = Self> {
        let rtic = RticBackend::value_variants()
            .iter()
            .copied()
            .map(|backend| Package::Rtic { backend });
        let mono = RticMonotonicsBackend::value_variants()
            .iter()
            .copied()
            .map(|backend| Package::RticMonotonics { backend });
        let macros = RticMacrosBackend::value_variants()
            .iter()
            .copied()
            .map(|backend| Package::RticMacros { backend });

        [Package::RticCommon, Package::RticSync, Package::RticTime]
            .into_iter()
            .chain(rtic)
            .chain(mono)
            .chain(macros)
    }
}

#[derive(clap::ValueEnum, Copy, Clone, Debug, PartialEq, Default)]
pub enum RticMonotonicsBackend {
    RiscvEsp32c3,
    #[default]
    Arm,
}

impl RticMonotonicsBackend {
    pub fn target(&self) -> Target {
        match self {
            RticMonotonicsBackend::RiscvEsp32c3 => RISCV32IMC,
            RticMonotonicsBackend::Arm => ARMV7M,
        }
    }

    pub fn features(&self, partial: bool) -> Vec<String> {
        let features = match self {
            RticMonotonicsBackend::RiscvEsp32c3 => &["esp32c3-systimer"][..],
            RticMonotonicsBackend::Arm => {
                if partial {
                    &[
                        "cortex-m-systick",
                        "rp2040",
                        "nrf52840",
                        "imxrt_gpt1,imxrt-ral/imxrt1062",
                        "stm32_tim2,stm32-metapac/stm32h725ag",
                    ][..]
                } else {
                    &[
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
                        "nrf9160-ns",
                        "nrf9160-s",
                        "nrf9151-ns",
                        "nrf9151-s",
                        "nrf9161-ns",
                        "nrf9161-s",
                        "imxrt_gpt1,imxrt_gpt2,imxrt-ral/imxrt1062",
                        "stm32_tim2,stm32_tim3,stm32_tim4,stm32_tim5,stm32_tim15,stm32-metapac/stm32h725ag",
                    ][..]
                }
            }
        };

        features.iter().map(|v| v.to_string()).collect()
    }
}

#[derive(clap::ValueEnum, Copy, Clone, Debug, PartialEq)]
pub enum RticMacrosBackend {
    CortexMSourceMasking,
    CortexMBasepri,
    RiscvEsp32c3,
    RiscvEsp32c6,
    RiscvClint,
    RiscvMecall,
}

impl RticMacrosBackend {
    pub fn feature(&self) -> String {
        let feature = match self {
            RticMacrosBackend::CortexMSourceMasking => "cortex-m-source-masking",
            RticMacrosBackend::CortexMBasepri => "cortex-m-basepri",
            RticMacrosBackend::RiscvEsp32c3 => "riscv-esp32c3",
            RticMacrosBackend::RiscvEsp32c6 => "riscv-esp32c6",
            RticMacrosBackend::RiscvClint => "riscv-clint",
            RticMacrosBackend::RiscvMecall => "riscv-mecall",
        };

        feature.to_owned()
    }
}

#[derive(clap::ValueEnum, Copy, Clone, Debug, PartialEq)]
pub enum RticBackend {
    Thumbv6,
    Thumbv7,
    Thumbv8Base,
    Thumbv8Main,
    RiscvEsp32c3,
    RiscvEsp32c6,
    RiscvImcClint,
    RiscvImcMecall,
    RiscvImacClint,
    RiscvImacMecall,
}

impl RticBackend {
    fn target(&self) -> Target {
        match self {
            RticBackend::Thumbv6 => ARMV6M,
            RticBackend::Thumbv7 => ARMV7M,
            RticBackend::Thumbv8Base => ARMV8MBASE,
            RticBackend::Thumbv8Main => ARMV8MMAIN,
            RticBackend::RiscvImcClint
            | RticBackend::RiscvImcMecall
            | RticBackend::RiscvEsp32c3 => RISCV32IMC,
            RticBackend::RiscvImacClint
            | RticBackend::RiscvImacMecall
            | RticBackend::RiscvEsp32c6 => RISCV32IMAC,
        }
    }

    /// Get the RTIC features required to build `rtic` for
    /// this backend.
    pub fn rtic_feature(&self) -> &'static str {
        match self {
            RticBackend::Thumbv6 => "rtic/thumbv6-backend",
            RticBackend::Thumbv7 => "rtic/thumbv7-backend",
            RticBackend::Thumbv8Base => "rtic/thumbv8base-backend",
            RticBackend::Thumbv8Main => "rtic/thumbv8main-backend",
            RticBackend::RiscvEsp32c3 => "rtic/riscv-esp32c3-backend",
            RticBackend::RiscvEsp32c6 => "rtic/riscv-esp32c6-backend",
            RticBackend::RiscvImcClint | RticBackend::RiscvImacClint => "rtic/riscv-clint-backend",
            RticBackend::RiscvImcMecall | RticBackend::RiscvImacMecall => {
                "rtic/riscv-mecall-backend"
            }
        }
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub enum BuildOrCheck {
    #[default]
    Check,
    Build,
}

impl Platform {
    pub fn name(&self) -> String {
        let name = match self {
            Platform::Esp32c3 => "esp32c3",
            Platform::Esp32c6 => "esp32c6",
            Platform::Hifive1 { .. } => "hifive1",
            Platform::Lm3s6965 { .. } => "lm3s6965",
            Platform::Nrf52840 => "nrf52840",
            Platform::Rp2040 => "rp2040",
            Platform::Stm32f3 => "stm32f3",
            Platform::Stm32f411 => "stm32f411",
            Platform::Teensy4 => "teensy4",
        };
        name.to_string()
    }
}

/// A single platform. If unspecified, all backends for
/// a platform are selected.
#[derive(Debug, Clone, Subcommand)]
pub enum PlatformSelector {
    Esp32c3,
    Esp32c6,
    Hifive1 {
        /// Which backends to compile for. If not specified, all
        /// backends are checked.
        #[clap(long, short, value_delimiter = ',')]
        backend: Vec<Hifive1Backend>,
    },
    Lm3s6965 {
        /// Which backends to compile for. If not specified, all
        /// backends are checked.
        #[clap(long, short, value_delimiter = ',')]
        backend: Vec<Lm3s6965Backend>,
    },
    Nrf52840,
    Rp2040,
    Stm32f3,
    Stm32f411,
    Teensy4,
}

impl PlatformSelector {
    fn platforms(&self) -> Vec<Platform> {
        let single = match self {
            PlatformSelector::Esp32c3 => Platform::Esp32c3,
            PlatformSelector::Esp32c6 => Platform::Esp32c6,
            PlatformSelector::Hifive1 { backend } => {
                return if backend.is_empty() {
                    Platform::all_hifive1().collect()
                } else {
                    backend
                        .iter()
                        .map(|b| Platform::Hifive1 { backend: *b })
                        .collect()
                };
            }
            PlatformSelector::Lm3s6965 { backend } => {
                return if backend.is_empty() {
                    Platform::all_hifive1().collect()
                } else {
                    backend
                        .iter()
                        .map(|b| Platform::Lm3s6965 { backend: *b })
                        .collect()
                };
            }
            PlatformSelector::Nrf52840 => Platform::Nrf52840,
            PlatformSelector::Rp2040 => Platform::Rp2040,
            PlatformSelector::Stm32f3 => Platform::Stm32f3,
            PlatformSelector::Stm32f411 => Platform::Stm32f411,
            PlatformSelector::Teensy4 => Platform::Teensy4,
        };

        vec![single]
    }
}

#[derive(Debug, Clone, Copy, Subcommand)]
pub enum Platform {
    Esp32c3,
    Esp32c6,
    Hifive1 {
        /// Which backend to compile for.
        #[clap(long, short)]
        backend: Hifive1Backend,
    },
    Lm3s6965 {
        /// Which backend to compile for.
        #[clap(long, short)]
        backend: Lm3s6965Backend,
    },
    Nrf52840,
    Rp2040,
    Stm32f3,
    Stm32f411,
    Teensy4,
}

impl Platform {
    /// Get the features required to build the example for this
    /// platform.
    pub fn example_features(&self) -> Option<String> {
        match self {
            Platform::Hifive1 { backend } => match backend {
                Hifive1Backend::Mecall => Some("riscv-mecall-backend".to_string()),
                Hifive1Backend::Clint => Some("riscv-clint-backend".to_string()),
            },
            _ => None,
        }
    }

    pub fn target(&self) -> Target {
        match self {
            Platform::Lm3s6965 { backend } => match backend {
                Lm3s6965Backend::Thumbv6 => ARMV6M,
                Lm3s6965Backend::Thumbv7 => ARMV7M,
                Lm3s6965Backend::Thumbv8Base => ARMV8MBASE,
                Lm3s6965Backend::Thumbv8Main => ARMV8MMAIN,
            },
            Platform::Esp32c3 => RISCV32IMC,
            Platform::Esp32c6 => RISCV32IMAC,
            Platform::Hifive1 { .. } => RISCV32IMC,
            Platform::Nrf52840 => ARMV7M,
            Platform::Rp2040 => ARMV6M,
            Platform::Stm32f3 => ARMV7M,
            Platform::Stm32f411 => ARMV7M,
            Platform::Teensy4 => ARMV7M,
        }
    }

    fn all() -> impl Iterator<Item = Self> {
        [
            Platform::Esp32c3,
            Platform::Esp32c6,
            // TODO: these don't map to example directories nicely, so we
            // cannot actually build their examples.
            // PlatformSelector::Nrf52840,
            // PlatformSelector::Stm32f3,
            // PlatformSelector::Stm32f411,
            // PlatformSelector::Teensy4,
            // PlatformSelector::Rp2040,
        ]
        .into_iter()
        .chain(Self::all_lm3s6965())
        .chain(Self::all_hifive1())
    }

    fn all_lm3s6965() -> impl Iterator<Item = Self> {
        Lm3s6965Backend::value_variants()
            .iter()
            .copied()
            .map(|backend| Platform::Lm3s6965 { backend })
    }

    fn all_hifive1() -> impl Iterator<Item = Self> {
        Hifive1Backend::value_variants()
            .iter()
            .copied()
            .map(|backend| Platform::Hifive1 { backend })
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Lm3s6965Backend {
    Thumbv6,
    Thumbv7,
    Thumbv8Base,
    Thumbv8Main,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Hifive1Backend {
    Mecall,
    Clint,
}

#[derive(Debug, Parser, Clone, Default)]
pub struct ExampleArgs {
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
}

impl ExampleArgs {
    pub fn get_examples(&self, platform: &Platform) -> anyhow::Result<Vec<String>> {
        let Self {
            example,
            exampleexclude,
        } = self;

        let examples_path = format!("./examples/{}/examples", platform.name());
        let examples: Vec<_> = std::fs::read_dir(examples_path)?
            .filter_map(|p| p.ok())
            .map(|p| p.path())
            .filter(|p| p.display().to_string().ends_with(".rs"))
            .map(|path| path.file_stem().unwrap().to_str().unwrap().to_string())
            .collect();

        let mut examples_to_run = examples.clone();

        if let Some(example) = example {
            examples_to_run = examples.clone();
            let examples_to_exclude = example.split(',').collect::<Vec<&str>>();
            // From the list of all examples, remove all not listed as included
            for ex in examples_to_exclude {
                examples_to_run.retain(|x| *x.as_str() == *ex);
            }
        };

        if let Some(example) = exampleexclude {
            examples_to_run = examples.clone();
            let examples_to_exclude = example.split(',').collect::<Vec<&str>>();
            // From the list of all examples, remove all those listed as excluded
            for ex in examples_to_exclude {
                examples_to_run.retain(|x| *x.as_str() != *ex);
            }
        };

        log::trace!("All examples:\n{examples:?} number: {}", examples.len());
        log::trace!(
            "examples_to_run:\n{examples_to_run:?} number: {}",
            examples_to_run.len()
        );

        if examples_to_run.is_empty() {
            log::error!(
                "\nThe example(s) you specified is not available. Available examples are:\
                    \n{examples:#?}\n\
             By default if example flag is emitted, all examples are tested.",
            );
            anyhow::bail!("Incorrect usage");
        } else {
            Ok(examples_to_run)
        }
    }
}

#[derive(Parser, Clone)]
pub struct Globals {
    /// Error out on warnings
    #[arg(short = 'D', long, global = true)]
    pub deny_warnings: bool,

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
    /// Run all of the commands that the CI also runs, but without
    /// setting global options. The global options are inherited from
    /// the caller, i.e. you.
    Ci {
        /// Skip the QEMU tests.
        #[clap(long, short = 'q')]
        skip_qemu: bool,
    },

    /// Format code
    #[clap(alias = "fmt")]
    Format(FormatOpt),

    /// Run clippy
    Clippy(PackageOpt),

    /// Check all packages
    Check(PackageOpt),

    /// Build all packages
    Build(PackageOpt),

    /// Run cargo check on selected or all examples.
    ExampleCheck {
        #[clap(flatten)]
        examples: ExampleArgs,
        #[clap(flatten)]
        selector: OptionalPlatformSelector,
    },

    /// Build selected or all examples.
    ExampleBuild {
        #[clap(flatten)]
        examples: ExampleArgs,
        #[clap(flatten)]
        selector: OptionalPlatformSelector,
    },

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
    Doc(ExtraArguments),

    /// Run tests
    Test(PackageOpt),

    /// Run rtic-sync loom tests
    TestLoom,

    /// Build books with mdbook
    Book(ExtraArguments),
}

#[derive(Args, Debug, Clone, Default)]
pub struct CiOpt {
    #[clap(short, long)]
    pub failearly: bool,
}

#[derive(Args, Debug, Clone)]
pub struct FormatOpt {
    /// Check-only, do not apply formatting fixes.
    #[clap(short, long)]
    pub check: bool,
}

#[derive(Args, Debug, Clone, Copy, Default)]
/// Restrict to package, or run on whole workspace
pub struct PackageOpt {
    #[clap(subcommand)]
    package: Option<Package>,
}

impl PackageOpt {
    /// A [`PackageOpt`] that will resolve to all packages.
    pub fn all() -> Self {
        Self::default()
    }

    /// A [`PackageOpt`] that will resolve to a single package.
    pub fn one(package: Package) -> Self {
        Self {
            package: Some(package),
        }
    }

    pub fn rtic_sync() -> Self {
        Self {
            package: Some(Package::RticSync),
        }
    }

    pub fn packages(&self) -> impl Iterator<Item = Package> {
        let pkgs = if let Some(package) = self.package {
            vec![package]
        } else {
            Package::all().collect()
        };

        pkgs.into_iter()
    }
}

#[derive(Args, Debug, Clone)]
pub struct QemuAndRun {
    #[clap(flatten)]
    pub examples: ExampleArgs,
    #[clap(subcommand)]
    pub target: Platform,
    /// If expected output is missing or mismatching, recreate the file
    ///
    /// This overwrites only missing or mismatching
    #[arg(long, global = true)]
    pub overwrite_expected: bool,
}

#[derive(Debug, Parser, Clone)]
pub struct ArgsAndOverwrite {
    #[clap(flatten)]
    pub examples: ExampleArgs,

    #[clap(flatten)]
    pub target: OptionalPlatformSelector,

    /// If expected output is missing or mismatching, recreate the file
    ///
    /// This overwrites only missing or mismatching
    #[arg(long, global = true)]
    pub overwrite_expected: bool,

    /// Options to pass to `cargo <subcommand>`
    #[command(flatten)]
    pub arguments: ExtraArguments,
}

#[derive(Debug, Clone, Parser, Default)]
pub struct OptionalPlatformSelector {
    /// The platform selector.
    ///
    /// If unspecified, all platforms are checked.
    ///
    /// Specifying a single platform will check all of its backends, or only
    /// those that are passed using `--backend`.
    #[clap(subcommand)]
    platform: Option<PlatformSelector>,
}

impl OptionalPlatformSelector {
    pub fn all() -> Self {
        Self::default()
    }

    pub fn platforms(&self) -> Vec<Platform> {
        if let Some(platform) = &self.platform {
            platform.platforms()
        } else {
            Platform::all().collect()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Parser, Default)]
pub struct ExtraArguments {
    /// All remaining flags and options
    pub rest: Vec<String>,
}
