use crate::{cargo_command::CargoCommand, Target, ARMV6M, ARMV7M, ARMV8MBASE, ARMV8MMAIN};
use clap::{Args, Parser, Subcommand};
use core::fmt;
use log::{error, trace};

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
                let features = if partial {
                    &["cortex-m-systick", "rp2040", "nrf52840"][..]
                } else {
                    &[
                        "cortex-m-systick,embedded-hal-async",
                        "cortex-m-systick,systick-100hz,embedded-hal-async",
                        "cortex-m-systick,systick-10khz,embedded-hal-async",
                        "rp2040,embedded-hal-async",
                        "nrf52810,embedded-hal-async",
                        "nrf52811,embedded-hal-async",
                        "nrf52832,embedded-hal-async",
                        "nrf52833,embedded-hal-async",
                        "nrf52840,embedded-hal-async",
                        "nrf5340-app,embedded-hal-async",
                        "nrf5340-net,embedded-hal-async",
                        "nrf9160,embedded-hal-async",
                    ][..]
                };

                features
                    .into_iter()
                    .map(ToString::to_string)
                    .map(Some)
                    .chain(std::iter::once(None))
                    .collect()
            }
            _ => vec![None],
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
                    package: Some(package.name()),
                    features,
                    test: Some("ui".to_owned()),
                    deny_warnings: true,
                }
            }
            Package::RticMacros => CargoCommand::Test {
                package: Some(package.name()),
                features: Some(backend.to_rtic_macros_feature().to_owned()),
                test: None,
                deny_warnings: true,
            },
            Package::RticSync => CargoCommand::Test {
                package: Some(package.name()),
                features: Some("testing".to_owned()),
                test: None,
                deny_warnings: true,
            },
            Package::RticCommon => CargoCommand::Test {
                package: Some(package.name()),
                features: Some("testing".to_owned()),
                test: None,
                deny_warnings: true,
            },
            Package::RticMonotonics => CargoCommand::Test {
                package: Some(package.name()),
                features: None,
                test: None,
                deny_warnings: true,
            },
            Package::RticTime => CargoCommand::Test {
                package: Some(package.name()),
                features: Some("critical-section/std".into()),
                test: None,
                deny_warnings: true,
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
    pub fn to_target(&self) -> Target<'static> {
        match self {
            Backends::Thumbv6 => ARMV6M,
            Backends::Thumbv7 => ARMV7M,
            Backends::Thumbv8Base => ARMV8MBASE,
            Backends::Thumbv8Main => ARMV8MMAIN,
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_rtic_feature(&self) -> &'static str {
        match self {
            Backends::Thumbv6 => "thumbv6-backend",
            Backends::Thumbv7 => "thumbv7-backend",
            Backends::Thumbv8Base => "thumbv8base-backend",
            Backends::Thumbv8Main => "thumbv8main-backend",
        }
    }
    #[allow(clippy::wrong_self_convention)]
    pub fn to_rtic_macros_feature(&self) -> &'static str {
        match self {
            Backends::Thumbv6 | Backends::Thumbv8Base => "cortex-m-source-masking",
            Backends::Thumbv7 | Backends::Thumbv8Main => "cortex-m-basepri",
        }
    }
    #[allow(clippy::wrong_self_convention)]
    pub fn to_rtic_uitest_feature(&self) -> &'static str {
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
    /// Error out on warnings
    #[arg(short = 'D', long)]
    pub deny_warnings: bool,

    /// For which backend to build.
    #[arg(value_enum, short, default_value = "thumbv7", long, global = true)]
    backend: Option<Backends>,

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

impl Globals {
    pub fn backend(&self) -> Backends {
        self.backend.unwrap_or_default()
    }
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
    ExampleCheck(ExampleArgs),

    /// Build all examples
    ExampleBuild(ExampleArgs),

    /// Run `cargo size` on selected or all examples
    ///
    /// To pass options to `cargo size`, add `--` and then the following
    /// arguments will be passed on
    ///
    /// Example: `cargo xtask size -- -A`
    Size(ExampleArgs),

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
    Doc(DocArgs),

    /// Run tests
    Test(PackageOpt),

    /// Build books with mdbook
    Book(BookArgs),

    /// Check one or more usage examples.
    ///
    /// Usage examples are located in ./examples
    UsageExampleCheck(UsageExamplesOpt),

    /// Build one or more usage examples.
    ///
    /// Usage examples are located in ./examples
    #[clap(alias = "./examples")]
    UsageExampleBuild(UsageExamplesOpt),
}

#[derive(Args, Clone, Debug)]
pub struct ExampleArgs {
    /// List of comma separated examples to include, all others are excluded
    ///
    /// If omitted all examples are included
    ///
    /// Example: `cargo xtask --example complex,spawn,init`
    /// would include complex, spawn and init
    #[arg(short, long, group = "example_group", global = true)]
    example: Option<String>,

    /// List of comma separated examples to exclude, all others are included
    ///
    /// If omitted all examples are included
    ///
    /// Example: `cargo xtask --excludeexample complex,spawn,init`
    /// would exclude complex, spawn and init
    #[arg(long, group = "example_group", global = true)]
    exampleexclude: Option<String>,

    /// Additional arguments to pass to the invoked tool
    #[command(subcommand)]
    pub arguments: Option<ExtraArguments>,
}

impl ExampleArgs {
    pub fn example_list(&self) -> anyhow::Result<Vec<String>> {
        let examples: Vec<_> = std::fs::read_dir("./rtic/examples")?
            .filter_map(|p| p.ok())
            .map(|p| p.path())
            .filter(|p| p.display().to_string().ends_with(".rs"))
            .map(|path| path.file_stem().unwrap().to_str().unwrap().to_string())
            .collect();

        let mut to_run = examples.clone();

        if let Some(example) = &self.example {
            to_run = examples.clone();
            let examples_to_exclude = example.split(',').collect::<Vec<&str>>();
            // From the list of all examples, remove all not listed as included
            for ex in examples_to_exclude {
                to_run.retain(|x| *x.as_str() == *ex);
            }
        };

        if let Some(example) = &self.exampleexclude {
            to_run = examples.clone();
            let examples_to_exclude = example.split(',').collect::<Vec<&str>>();
            // From the list of all examples, remove all those listed as excluded
            for ex in examples_to_exclude {
                to_run.retain(|x| *x.as_str() != *ex);
            }
        };

        trace!("All examples (N: {}): {examples:?}", examples.len());
        trace!("Examples to run (N: {}): {to_run:?}", to_run.len());

        if to_run.is_empty() {
            error!("The example(s) you specified cannot be found. Available examples are:");
            error!("{examples:#?}");
            error!("");
            error!("By default, all examples are tested");
            Err(anyhow::anyhow!("Incorrect usage"))
        } else {
            Ok(to_run)
        }
    }
}

#[derive(Args, Clone, Debug)]
pub struct BookArgs {
    /// If this flag is set, the links in the book are
    /// not verified
    #[clap(long, short = 'l')]
    pub skip_link_check: bool,

    /// If this flag is set, the links in the API documentation
    /// included with the book are not verified
    #[clap(long, short = 'a')]
    pub skip_api_link_check: bool,

    /// The path to which the contents of the book should
    /// be written.
    #[clap(long, short, default_value = "book-target")]
    pub output_path: String,

    /// Additional arguments to pass to `mdbook`
    #[command(subcommand)]
    pub arguments: Option<ExtraArguments>,
}

#[derive(Args, Clone, Debug)]
pub struct DocArgs {
    /// If this flag is set, the links in the API
    /// docs are not verified.
    #[clap(long, short = 'l')]
    pub skip_link_check: bool,

    /// Additional arguments to pass to `cargo doc`
    #[command(subcommand)]
    pub arguments: Option<ExtraArguments>,
}

#[derive(Args, Clone, Debug)]
pub struct UsageExamplesOpt {
    /// The usage examples to build. All usage examples are selected if this argument is not provided.
    ///
    /// Example: `rp2040_local_i2c_init,stm32f3_blinky`.
    examples: Option<String>,
}

#[derive(Args, Clone, Debug)]
pub struct BookOpt {
    /// The directory in which the book is located
    #[clap(default_value = "./book/en", short, long)]
    pub directory: String,
    /// The directory to put the final book in.
    #[clap(default_value = "./book-target", short, long)]
    pub output_dir: String,
}

impl UsageExamplesOpt {
    pub fn examples(&self) -> anyhow::Result<Vec<String>> {
        let usage_examples: Vec<_> = std::fs::read_dir("./examples")?
            .filter_map(Result::ok)
            .filter(|p| p.metadata().ok().map(|p| p.is_dir()).unwrap_or(false))
            .filter_map(|p| p.file_name().to_str().map(ToString::to_string))
            .collect();

        let selected_examples: Option<Vec<String>> = self
            .examples
            .clone()
            .map(|s| s.split(",").map(ToString::to_string).collect());

        if let Some(selected_examples) = selected_examples {
            if let Some(unfound_example) = selected_examples
                .iter()
                .find(|e| !usage_examples.contains(e))
            {
                Err(anyhow::anyhow!(
                    "Usage example {unfound_example} does not exist"
                ))
            } else {
                Ok(selected_examples)
            }
        } else {
            Ok(usage_examples)
        }
    }
}

#[derive(Args, Debug, Clone)]
pub struct FormatOpt {
    #[clap(flatten)]
    pub package: PackageOpt,
    /// Check-only, do not apply formatting fixes.
    #[clap(short, long)]
    pub check: bool,
}

#[derive(Args, Debug, Clone)]
/// Restrict to package, or run on whole workspace
pub struct PackageOpt {
    /// Don't build/check/test all feature combinations that are available, only
    /// a necessary subset.
    #[arg(long)]
    pub partial: bool,

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
    #[clap(flatten)]
    pub examples: ExampleArgs,

    /// If expected output is missing or mismatching, recreate the file
    ///
    /// This overwrites only missing or mismatching
    #[arg(long)]
    pub overwrite_expected: bool,
}

#[derive(Debug, Parser, Clone)]
pub struct Arg {
    /// Additional arguments to pass to called binaries.
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
