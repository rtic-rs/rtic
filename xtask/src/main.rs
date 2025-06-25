mod argument_parsing;
mod build;
mod cargo_command;
mod run;

use argument_parsing::{ExtraArguments, FormatOpt, Package, PackageOpt, TestOpt};
use clap::Parser;
use core::fmt;
use std::{path::Path, str};

use log::{error, log_enabled, trace, Level};

use crate::{
    argument_parsing::{BuildOrCheck, Cli, Commands, Platforms},
    build::init_build_dir,
    run::*,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Target<'a> {
    triple: &'a str,
    has_std: bool,
}

impl<'a> Target<'a> {
    const DEFAULT_FEATURES: &'static str = "test-critical-section";

    pub const fn new(triple: &'a str, has_std: bool) -> Self {
        Self { triple, has_std }
    }

    pub fn triple(&self) -> &str {
        self.triple
    }

    pub fn has_std(&self) -> bool {
        self.has_std
    }

    pub fn and_features(&self, features: &str) -> String {
        format!("{},{}", Self::DEFAULT_FEATURES, features)
    }
}

impl core::fmt::Display for Target<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.triple)
    }
}

// x86_64-unknown-linux-gnu
const _X86_64: Target = Target::new("x86_64-unknown-linux-gnu", true);
const ARMV6M: Target = Target::new("thumbv6m-none-eabi", false);
const ARMV7M: Target = Target::new("thumbv7m-none-eabi", false);
const ARMV8MBASE: Target = Target::new("thumbv8m.base-none-eabi", false);
const ARMV8MMAIN: Target = Target::new("thumbv8m.main-none-eabi", false);
const RISCV32IMC: Target = Target::new("riscv32imc-unknown-none-elf", false);
const RISCV32IMAC: Target = Target::new("riscv32imac-unknown-none-elf", false);

fn main() -> anyhow::Result<()> {
    // if there's an `xtask` folder, we're *probably* at the root of this repo (we can't just
    // check the name of `env::current_dir()` because people might clone it into a different name)
    let probably_running_from_repo_root = Path::new("./xtask").exists();
    if !probably_running_from_repo_root {
        return Err(anyhow::anyhow!(
            "xtasks can only be executed from the root of the `rtic` repository"
        ));
    }

    let cli = Cli::parse();

    let globals = &cli.globals;

    let env_logger_default_level = match globals.verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };

    pretty_env_logger::formatted_builder()
        .parse_filters(&std::env::var("RUST_LOG").unwrap_or(env_logger_default_level.into()))
        .init();

    trace!("default logging level: {0}", globals.verbose);

    log::debug!(
        "Stderr of child processes is inherited: {}",
        globals.stderr_inherited
    );
    log::debug!("Partial features: {}", globals.partial);

    let platform = globals.platform.unwrap_or_default();

    let backend = if let Some(backend) = globals.backend {
        backend
    } else {
        platform.default_backend()
    };

    // Check if the platform supports the backend
    if platform.features(&backend).is_err() {
        return Err(anyhow::anyhow!(
            "platform {:?} does not support backend {:?}",
            platform,
            backend
        ));
    }
    let examples_path = format!("./examples/{}/examples", platform.name());
    let examples: Vec<_> = std::fs::read_dir(examples_path)?
        .filter_map(|p| p.ok())
        .map(|p| p.path())
        .filter(|p| p.display().to_string().ends_with(".rs"))
        .map(|path| path.file_stem().unwrap().to_str().unwrap().to_string())
        .collect();

    let example = globals.example.clone();
    let exampleexclude = globals.exampleexclude.clone();

    let examples_to_run = {
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

        if log_enabled!(Level::Trace) {
            trace!("All examples:\n{examples:?} number: {}", examples.len());
            trace!(
                "examples_to_run:\n{examples_to_run:?} number: {}",
                examples_to_run.len()
            );
        }

        if examples_to_run.is_empty() {
            error!(
                "\nThe example(s) you specified is not available. Available examples are:\
                    \n{examples:#?}\n\
             By default if example flag is emitted, all examples are tested.",
            );
            return Err(anyhow::anyhow!("Incorrect usage"));
        } else {
            examples_to_run
        }
    };

    init_build_dir()?;
    #[allow(clippy::if_same_then_else)]
    let cargoarg = if log_enabled!(Level::Trace) {
        Some("-v")
    } else if log_enabled!(Level::Debug) {
        None
    } else if log_enabled!(Level::Info) {
        None
    } else if log_enabled!(Level::Warn) || log_enabled!(Level::Error) {
        None
    } else {
        // Off case
        Some("--quiet")
    };

    let formatoptcheckonly = FormatOpt {
        // Only check, do not reformat
        check: true,
        ..Default::default()
    };

    // Default set of all packages
    // CI always runs on all packages
    let package = PackageOpt::default();
    let testopts = TestOpt::default();
    // Currently only rtic-sync supports loom tests
    let testoptsloom = TestOpt {
        loom: true,
        package: PackageOpt {
            package: Some(Package::RticSync),
        },
    };

    let final_run_results = match &cli.command {
        Commands::AllCi(args) => {
            // TODO: Reduce code duplication and repetition
            let mut results = cargo_format(globals, &cargoarg, &formatoptcheckonly);
            if args.failearly {
                return handle_results(globals, results)
                    .map_err(|_| anyhow::anyhow!("Commands failed"));
            }

            results.append(&mut cargo_clippy(globals, &cargoarg, &package, backend));

            if args.failearly {
                return handle_results(globals, results)
                    .map_err(|_| anyhow::anyhow!("Commands failed"));
            }

            results.append(&mut cargo(
                globals,
                BuildOrCheck::Check,
                &cargoarg,
                &package,
                backend,
            ));
            if args.failearly {
                return handle_results(globals, results)
                    .map_err(|_| anyhow::anyhow!("Commands failed"));
            }
            results.append(&mut cargo(
                globals,
                BuildOrCheck::Build,
                &cargoarg,
                &package,
                backend,
            ));
            if args.failearly {
                return handle_results(globals, results)
                    .map_err(|_| anyhow::anyhow!("Commands failed"));
            }

            results.append(&mut cargo_example(
                globals,
                BuildOrCheck::Check,
                &cargoarg,
                platform,
                backend,
                &examples_to_run,
            ));
            if args.failearly {
                return handle_results(globals, results)
                    .map_err(|_| anyhow::anyhow!("Commands failed"));
            }
            results.append(&mut cargo_example(
                globals,
                BuildOrCheck::Build,
                &cargoarg,
                platform,
                backend,
                &examples_to_run,
            ));
            if args.failearly {
                return handle_results(globals, results)
                    .map_err(|_| anyhow::anyhow!("Commands failed"));
            }
            results.append(&mut qemu_run_examples(
                globals,
                &cargoarg,
                platform,
                backend,
                &examples_to_run,
                false,
            ));
            if args.failearly {
                return handle_results(globals, results)
                    .map_err(|_| anyhow::anyhow!("Commands failed"));
            }

            results.append(&mut cargo_doc(globals, &cargoarg, backend, &None));
            if args.failearly {
                return handle_results(globals, results)
                    .map_err(|_| anyhow::anyhow!("Commands failed"));
            }
            results.append(&mut cargo_test(globals, &testopts, backend));
            if args.failearly {
                return handle_results(globals, results)
                    .map_err(|_| anyhow::anyhow!("Commands failed"));
            }
            results.append(&mut cargo_test(globals, &testoptsloom, backend));
            if args.failearly {
                return handle_results(globals, results)
                    .map_err(|_| anyhow::anyhow!("Commands failed"));
            }
            results.append(&mut cargo_book(globals, &None));
            if args.failearly {
                return handle_results(globals, results)
                    .map_err(|_| anyhow::anyhow!("Commands failed"));
            }

            results
        }
        Commands::Format(formatopts) => cargo_format(globals, &cargoarg, formatopts),
        Commands::Clippy(packageopts) => cargo_clippy(globals, &cargoarg, packageopts, backend),
        Commands::Check(args) => cargo(globals, BuildOrCheck::Check, &cargoarg, args, backend),
        Commands::Build(args) => cargo(globals, BuildOrCheck::Build, &cargoarg, args, backend),
        Commands::ExampleCheck => cargo_example(
            globals,
            BuildOrCheck::Check,
            &cargoarg,
            platform,
            backend,
            &examples_to_run,
        ),
        Commands::ExampleBuild => cargo_example(
            globals,
            BuildOrCheck::Build,
            &cargoarg,
            platform,
            backend,
            &examples_to_run,
        ),
        Commands::Size(args) => {
            // x86_64 target not valid
            build_and_check_size(
                globals,
                &cargoarg,
                platform,
                backend,
                &examples_to_run,
                args.overwrite_expected,
                &args.arguments,
            )
        }
        Commands::Qemu(args) | Commands::Run(args) => {
            // x86_64 target not valid
            qemu_run_examples(
                globals,
                &cargoarg,
                platform,
                backend,
                &examples_to_run,
                args.overwrite_expected,
            )
        }
        Commands::Doc(args) => cargo_doc(globals, &cargoarg, backend, &args.arguments),
        Commands::Test(args) => cargo_test(globals, args, backend),
        Commands::Book(args) => cargo_book(globals, &args.arguments),
    };

    handle_results(globals, final_run_results).map_err(|_| anyhow::anyhow!("Commands failed"))
}
