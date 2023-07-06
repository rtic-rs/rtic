mod argument_parsing;
mod build;
mod cargo_command;
mod run;

use argument_parsing::ExtraArguments;
use clap::Parser;
use core::fmt;
use std::{path::Path, str};

use log::{log_enabled, trace, Level};

use crate::{
    argument_parsing::{BuildOrCheck, Cli, Commands},
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
        Commands::Clippy(args) => cargo_clippy(globals, &cargologlevel, &args, args.partial),
        Commands::Check(args) => cargo(
            globals,
            BuildOrCheck::Check,
            &cargologlevel,
            &args,
            args.partial,
        ),
        Commands::Build(args) => cargo(
            globals,
            BuildOrCheck::Build,
            &cargologlevel,
            &args,
            args.partial,
        ),
        Commands::ExampleCheck(ex) => {
            let ex = ex.example_list()?;
            cargo_example(globals, BuildOrCheck::Check, &cargologlevel, ex)
        }
        Commands::ExampleBuild(ex) => {
            let ex = ex.example_list()?;
            cargo_example(globals, BuildOrCheck::Build, &cargologlevel, ex)
        }
        Commands::Size(args) => {
            // x86_64 target not valid
            let ex = args.example_list()?;
            build_and_check_size(globals, &cargologlevel, ex, &args.arguments)
        }
        Commands::Qemu(args) | Commands::Run(args) => {
            // x86_64 target not valid
            let ex = args.examples.example_list()?;
            qemu_run_examples(globals, &cargologlevel, ex, args.overwrite_expected)
        }
        Commands::Test(args) => cargo_test(globals, &args),
        Commands::Book(args) => {
            let links = !args.skip_link_check;
            let api_links = !args.skip_api_link_check;
            let output = std::path::PathBuf::from(&args.output_path);
            let api = args.api_docs.clone().map(std::path::PathBuf::from);
            cargo_book(globals, links, api_links, output, api, &args.arguments)
        }
        Commands::UsageExampleCheck(examples) => {
            cargo_usage_example(globals, BuildOrCheck::Check, examples.examples()?)
        }
        Commands::UsageExampleBuild(examples) => {
            cargo_usage_example(globals, BuildOrCheck::Build, examples.examples()?)
        }
    };

    handle_results(globals, final_run_results).map_err(|_| anyhow::anyhow!("Commands failed"))
}
