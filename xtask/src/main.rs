mod argument_parsing;
mod build;
mod cargo_command;
mod run;

use clap::Parser;
use core::fmt;
use std::{path::Path, str};

use log::{log_enabled, trace, Level};

use crate::{
    argument_parsing::{
        BuildOrCheck, Cli, Commands, ExampleArgs, ExtraArguments, FormatOpt, Globals,
        Hifive1Backend, OptionalPlatformSelector, Package, PackageOpt, Platform, QemuAndRun,
    },
    build::init_build_dir,
    run::*,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Target {
    triple: &'static str,
    has_std: bool,
    has_cas: bool,
}

impl Target {
    pub const fn new(triple: &'static str, has_std: bool, has_cas: bool) -> Self {
        Self {
            triple,
            has_std,
            has_cas,
        }
    }

    pub fn triple(&self) -> &'static str {
        self.triple
    }

    pub fn has_std(&self) -> bool {
        self.has_std
    }

    /// Get the feature needed to enable CAS for thsi target.
    pub fn cas_feature(&self) -> Option<String> {
        (!self.has_cas).then_some("portable-atomic/unsafe-assume-single-core".to_string())
    }
}

impl core::fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.triple)
    }
}

const ARMV6M: Target = Target::new("thumbv6m-none-eabi", false, false);
const ARMV7M: Target = Target::new("thumbv7m-none-eabi", false, true);
const ARMV8MBASE: Target = Target::new("thumbv8m.base-none-eabi", false, true);
const ARMV8MMAIN: Target = Target::new("thumbv8m.main-none-eabi", false, true);
const RISCV32IMC: Target = Target::new("riscv32imc-unknown-none-elf", false, false);
const RISCV32IMAC: Target = Target::new("riscv32imac-unknown-none-elf", false, true);

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

    let final_run_results = run_command(&cli.globals, cli.command, cargoarg)?;

    handle_results(globals, final_run_results).map_err(|_| anyhow::anyhow!("Commands failed"))
}

fn run_command(
    globals: &Globals,
    command: Commands,
    cargoarg: Option<&str>,
) -> anyhow::Result<Vec<FinalRunResult>> {
    let rtic_sync = PackageOpt::rtic_sync();
    let cargoarg = cargoarg.map(|v| v.to_owned());

    let results = match command {
        Commands::Ci { skip_qemu } => {
            use Commands::*;

            // Test command skips RTIC for RISCV32: the UI tests do not work. We
            // do still run clippy on them.
            let test = Package::all()
                .filter(|p| {
                    p.target()
                        .is_none_or(|t| t != RISCV32IMAC && t != RISCV32IMC)
                })
                .map(|p| Test(PackageOpt::one(p)));

            let mut commands = vec![
                Format(FormatOpt { check: true }),
                ExampleBuild {
                    examples: ExampleArgs::default(),
                    selector: OptionalPlatformSelector::all(),
                },
                Clippy(PackageOpt::all()),
                TestLoom,
                Doc(ExtraArguments::default()),
                // This one is hard to get right.
                // Book(Default::default()),
            ];

            if !skip_qemu {
                commands.extend([
                    Qemu(QemuAndRun {
                        examples: ExampleArgs::default(),
                        target: Platform::Lm3s6965 {
                            backend: argument_parsing::Lm3s6965Backend::Thumbv7,
                        },
                        overwrite_expected: false,
                    }),
                    Qemu(QemuAndRun {
                        examples: ExampleArgs::default(),
                        target: Platform::Lm3s6965 {
                            backend: argument_parsing::Lm3s6965Backend::Thumbv6,
                        },
                        overwrite_expected: false,
                    }),
                    Qemu(QemuAndRun {
                        examples: ExampleArgs::default(),
                        target: Platform::Hifive1 {
                            backend: Hifive1Backend::Clint,
                        },
                        overwrite_expected: false,
                    }),
                    Qemu(QemuAndRun {
                        examples: ExampleArgs::default(),
                        target: Platform::Hifive1 {
                            backend: Hifive1Backend::Mecall,
                        },
                        overwrite_expected: false,
                    }),
                ]);
            }

            commands.into_iter().chain(test).try_fold(
                Vec::new(),
                |mut acc, cmd| match run_command(globals, cmd, cargoarg.as_deref()) {
                    Ok(mut v) => {
                        acc.append(&mut v);
                        Ok(acc)
                    }
                    Err(e) => Err(e),
                },
            )?
        }
        Commands::Format(formatopts) => cargo_format(globals, cargoarg, &formatopts)?,
        Commands::Clippy(packageopts) => cargo_clippy(globals, cargoarg, packageopts),
        Commands::Check(args) => cargo(globals, BuildOrCheck::Check, cargoarg, args),
        Commands::Build(args) => cargo(globals, BuildOrCheck::Build, cargoarg, args),
        Commands::ExampleCheck {
            examples: args,
            selector,
        } => cargo_example(
            globals,
            BuildOrCheck::Check,
            cargoarg,
            selector.platforms(),
            args,
        ),
        Commands::ExampleBuild {
            examples: args,
            selector,
        } => cargo_example(
            globals,
            BuildOrCheck::Build,
            cargoarg,
            selector.platforms(),
            args,
        ),
        Commands::Size(args) => {
            // x86_64 target not valid
            build_and_check_size(
                globals,
                cargoarg,
                args.target.platforms(),
                &args.examples,
                args.overwrite_expected,
                &args.arguments.rest,
            )
        }
        Commands::Qemu(args) | Commands::Run(args) => {
            // x86_64 target not valid
            qemu_run_examples(
                globals,
                cargoarg,
                args.target,
                args.examples.get_examples(&args.target)?,
                args.overwrite_expected,
            )
        }
        Commands::Doc(args) => cargo_doc(globals, cargoarg, &args.rest),
        Commands::Test(args) => cargo_test(globals, args, false),
        Commands::TestLoom => cargo_test(globals, rtic_sync, true),
        Commands::Book(args) => cargo_book(globals, &args.rest),
    };

    Ok(results)
}
