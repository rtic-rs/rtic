mod argument_parsing;
mod build;
mod cargo_command;
mod run;

use argument_parsing::ExtraArguments;
use clap::Parser;
use core::fmt;
use std::{path::Path, str};

use log::{error, info, log_enabled, trace, Level};

use crate::{
    argument_parsing::{Backends, BuildOrCheck, Cli, Commands},
    build::init_build_dir,
    run::*,
};

#[derive(Debug, Clone, Copy)]
pub struct Target<'a> {
    triple: &'a str,
    has_std: bool,
}

impl<'a> Target<'a> {
    const DEFAULT_FEATURES: &str = "test-critical-section";

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

fn main() -> anyhow::Result<()> {
    // if there's an `xtask` folder, we're *probably* at the root of this repo (we can't just
    // check the name of `env::current_dir()` because people might clone it into a different name)
    let probably_running_from_repo_root = Path::new("./xtask").exists();
    if !probably_running_from_repo_root {
        return Err(anyhow::anyhow!(
            "xtasks can only be executed from the root of the `rtic` repository"
        ));
    }

    let examples: Vec<_> = std::fs::read_dir("./rtic/examples")?
        .filter_map(|p| p.ok())
        .map(|p| p.path())
        .filter(|p| p.display().to_string().ends_with(".rs"))
        .map(|path| path.file_stem().unwrap().to_str().unwrap().to_string())
        .collect();

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

    let backend = if let Some(backend) = globals.backend {
        backend
    } else {
        Backends::default()
    };

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
    let cargologlevel = if log_enabled!(Level::Trace) {
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

    let final_run_results = match &cli.command {
        Commands::Format(args) => cargo_format(globals, &cargologlevel, &args.package, args.check),
        Commands::Clippy(args) => {
            info!("Running clippy on backend: {backend:?}");
            cargo_clippy(globals, &cargologlevel, &args, backend)
        }
        Commands::Check(args) => {
            info!("Checking on backend: {backend:?}");
            cargo(globals, BuildOrCheck::Check, &cargologlevel, &args, backend)
        }
        Commands::Build(args) => {
            info!("Building for backend: {backend:?}");
            cargo(globals, BuildOrCheck::Build, &cargologlevel, &args, backend)
        }
        Commands::ExampleCheck => {
            info!("Checking on backend: {backend:?}");
            cargo_example(
                globals,
                BuildOrCheck::Check,
                &cargologlevel,
                backend,
                &examples_to_run,
            )
        }
        Commands::ExampleBuild => {
            info!("Building for backend: {backend:?}");
            cargo_example(
                globals,
                BuildOrCheck::Build,
                &cargologlevel,
                backend,
                &examples_to_run,
            )
        }
        Commands::Size(args) => {
            // x86_64 target not valid
            info!("Measuring for backend: {backend:?}");
            build_and_check_size(
                globals,
                &cargologlevel,
                backend,
                &examples_to_run,
                &args.arguments,
            )
        }
        Commands::Qemu(args) | Commands::Run(args) => {
            // x86_64 target not valid
            info!("Testing for backend: {backend:?}");
            qemu_run_examples(
                globals,
                &cargologlevel,
                backend,
                &examples_to_run,
                args.overwrite_expected,
            )
        }
        Commands::Doc(args) => {
            info!("Running cargo doc on backend: {backend:?}");
            cargo_doc(globals, &cargologlevel, backend, &args.arguments)
        }
        Commands::Test(args) => {
            info!("Running cargo test on backend: {backend:?}");
            cargo_test(globals, &args, backend)
        }
        Commands::Book(args) => {
            info!("Running mdbook");
            cargo_book(globals, &args.arguments)
        }
        Commands::UsageExampleCheck(examples) => {
            info!("Checking usage examples");
            cargo_usage_example(globals, BuildOrCheck::Check, examples.examples()?)
        }
        Commands::UsageExampleBuild(examples) => {
            info!("Building usage examples");
            cargo_usage_example(globals, BuildOrCheck::Build, examples.examples()?)
        }
    };

    handle_results(globals, final_run_results).map_err(|_| anyhow::anyhow!("Commands failed"))
}
