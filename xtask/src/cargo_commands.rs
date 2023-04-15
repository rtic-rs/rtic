use crate::{
    argument_parsing::{Backends, BuildOrCheck, ExtraArguments, Globals, PackageOpt, TestMetadata},
    command::{BuildMode, CargoCommand},
    command_parser,
};
use log::error;
use rayon::prelude::*;

/// Cargo command to either build or check
pub fn cargo(
    globals: &Globals,
    operation: BuildOrCheck,
    cargoarg: &Option<&str>,
    package: &PackageOpt,
    backend: Backends,
) -> anyhow::Result<()> {
    package.packages().for_each(|package| {
        let target = backend.to_target();

        let features = package.extract_features(target, backend);

        match operation {
            BuildOrCheck::Check => {
                log::debug!(target: "xtask::command", "Checking package: {package}")
            }
            BuildOrCheck::Build => {
                log::debug!(target: "xtask::command", "Building package: {package}")
            }
        }

        let command = match operation {
            BuildOrCheck::Check => CargoCommand::Check {
                cargoarg,
                package: Some(package),
                target,
                features,
                mode: BuildMode::Release,
            },
            BuildOrCheck::Build => CargoCommand::Build {
                cargoarg,
                package: Some(package),
                target,
                features,
                mode: BuildMode::Release,
            },
        };
        let res = command_parser(globals, &command, false);
        if let Err(e) = res {
            error!("{e}");
        }
    });

    Ok(())
}

/// Cargo command to either build or check all examples
///
/// The examples are in rtic/examples
pub fn cargo_example(
    globals: &Globals,
    operation: BuildOrCheck,
    cargoarg: &Option<&str>,
    backend: Backends,
    examples: &[String],
) -> anyhow::Result<()> {
    examples.into_par_iter().for_each(|example| {
        let features = Some(backend.to_target().and_features(backend.to_rtic_feature()));

        let command = match operation {
            BuildOrCheck::Check => CargoCommand::ExampleCheck {
                cargoarg,
                example,
                target: backend.to_target(),
                features,
                mode: BuildMode::Release,
            },
            BuildOrCheck::Build => CargoCommand::ExampleBuild {
                cargoarg,
                example,
                target: backend.to_target(),
                features,
                mode: BuildMode::Release,
            },
        };

        if let Err(err) = command_parser(globals, &command, false) {
            error!("{err}");
        }
    });

    Ok(())
}

/// Run cargo clippy on selected package
pub fn cargo_clippy(
    globals: &Globals,
    cargoarg: &Option<&str>,
    package: &PackageOpt,
    backend: Backends,
) -> anyhow::Result<()> {
    package.packages().for_each(|p| {
        let target = backend.to_target();
        let features = p.extract_features(target, backend);

        let res = command_parser(
            globals,
            &CargoCommand::Clippy {
                cargoarg,
                package: Some(p),
                target,
                features,
            },
            false,
        );

        if let Err(e) = res {
            error!("{e}")
        }
    });

    Ok(())
}

/// Run cargo fmt on selected package
pub fn cargo_format(
    globals: &Globals,
    cargoarg: &Option<&str>,
    package: &PackageOpt,
    check_only: bool,
) -> anyhow::Result<()> {
    package.packages().for_each(|p| {
        let res = command_parser(
            globals,
            &CargoCommand::Format {
                cargoarg,
                package: Some(p),
                check_only,
            },
            false,
        );

        if let Err(e) = res {
            error!("{e}")
        }
    });

    Ok(())
}

/// Run cargo doc
pub fn cargo_doc(
    globals: &Globals,
    cargoarg: &Option<&str>,
    backend: Backends,
    arguments: &Option<ExtraArguments>,
) -> anyhow::Result<()> {
    let features = Some(backend.to_target().and_features(backend.to_rtic_feature()));

    command_parser(
        globals,
        &CargoCommand::Doc {
            cargoarg,
            features,
            arguments: arguments.clone(),
        },
        false,
    )?;
    Ok(())
}

/// Run cargo test on the selected package or all packages
///
/// If no package is specified, loop through all packages
pub fn cargo_test(
    globals: &Globals,
    package: &PackageOpt,
    backend: Backends,
) -> anyhow::Result<()> {
    package.packages().for_each(|p| {
        let cmd = &TestMetadata::match_package(p, backend);
        if let Err(err) = command_parser(globals, cmd, false) {
            error!("{err}")
        }
    });

    Ok(())
}

/// Use mdbook to build the book
pub fn cargo_book(globals: &Globals, arguments: &Option<ExtraArguments>) -> anyhow::Result<()> {
    command_parser(
        globals,
        &CargoCommand::Book {
            arguments: arguments.clone(),
        },
        false,
    )?;
    Ok(())
}

/// Run examples
///
/// Supports updating the expected output via the overwrite argument
pub fn run_test(
    globals: &Globals,
    cargoarg: &Option<&str>,
    backend: Backends,
    examples: &[String],
    overwrite: bool,
) -> anyhow::Result<()> {
    let target = backend.to_target();
    let features = Some(target.and_features(backend.to_rtic_feature()));

    examples.into_par_iter().for_each(|example| {
        let cmd = CargoCommand::ExampleBuild {
            cargoarg: &Some("--quiet"),
            example,
            target,
            features: features.clone(),
            mode: BuildMode::Release,
        };

        if let Err(err) = command_parser(globals, &cmd, false) {
            error!("{err}");
        }

        let cmd = CargoCommand::Qemu {
            cargoarg,
            example,
            target,
            features: features.clone(),
            mode: BuildMode::Release,
        };

        if let Err(err) = command_parser(globals, &cmd, overwrite) {
            error!("{err}");
        }
    });

    Ok(())
}

/// Check the binary sizes of examples
pub fn build_and_check_size(
    globals: &Globals,
    cargoarg: &Option<&str>,
    backend: Backends,
    examples: &[String],
    arguments: &Option<ExtraArguments>,
) -> anyhow::Result<()> {
    let target = backend.to_target();
    let features = Some(target.and_features(backend.to_rtic_feature()));

    examples.into_par_iter().for_each(|example| {
        // Make sure the requested example(s) are built
        let cmd = CargoCommand::ExampleBuild {
            cargoarg: &Some("--quiet"),
            example,
            target,
            features: features.clone(),
            mode: BuildMode::Release,
        };
        if let Err(err) = command_parser(globals, &cmd, false) {
            error!("{err}");
        }

        let cmd = CargoCommand::ExampleSize {
            cargoarg,
            example,
            target: backend.to_target(),
            features: features.clone(),
            mode: BuildMode::Release,
            arguments: arguments.clone(),
        };
        if let Err(err) = command_parser(globals, &cmd, false) {
            error!("{err}");
        }
    });

    Ok(())
}
