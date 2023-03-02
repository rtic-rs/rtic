use crate::{
    argument_parsing::{Backends, BuildOrCheck, Package, PackageOpt, Sizearguments, TestMetadata},
    command::{BuildMode, CargoCommand},
    command_parser, package_feature_extractor, DEFAULT_FEATURES,
};
use log::error;
use rayon::prelude::*;

/// Cargo command to either build or check
pub fn cargo(
    operation: BuildOrCheck,
    cargoarg: &Option<&str>,
    package: &PackageOpt,
    backend: Backends,
) -> anyhow::Result<()> {
    let features = package_feature_extractor(package, backend);

    let command = match operation {
        BuildOrCheck::Check => CargoCommand::Check {
            cargoarg,
            package: package.package,
            target: backend.to_target(),
            features,
            mode: BuildMode::Release,
        },
        BuildOrCheck::Build => CargoCommand::Build {
            cargoarg,
            package: package.package,
            target: backend.to_target(),
            features,
            mode: BuildMode::Release,
        },
    };
    command_parser(&command, false)?;
    Ok(())
}

/// Cargo command to either build or check all examples
///
/// The examples are in rtic/examples
pub fn cargo_example(
    operation: BuildOrCheck,
    cargoarg: &Option<&str>,
    backend: Backends,
    examples: &[String],
) -> anyhow::Result<()> {
    examples.into_par_iter().for_each(|example| {
        let features = Some(format!(
            "{},{}",
            DEFAULT_FEATURES,
            backend.to_rtic_feature()
        ));

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

        if let Err(err) = command_parser(&command, false) {
            error!("{err}");
        }
    });

    Ok(())
}

/// Run cargo clippy on selected package
pub fn cargo_clippy(
    cargoarg: &Option<&str>,
    package: &PackageOpt,
    backend: Backends,
) -> anyhow::Result<()> {
    let features = package_feature_extractor(package, backend);
    command_parser(
        &CargoCommand::Clippy {
            cargoarg,
            package: package.package,
            target: backend.to_target(),
            features,
        },
        false,
    )?;
    Ok(())
}

/// Run cargo fmt on selected package
pub fn cargo_format(
    cargoarg: &Option<&str>,
    package: &PackageOpt,
    check_only: bool,
) -> anyhow::Result<()> {
    command_parser(
        &CargoCommand::Format {
            cargoarg,
            package: package.package,
            check_only,
        },
        false,
    )?;
    Ok(())
}

/// Run cargo doc
pub fn cargo_doc(cargoarg: &Option<&str>, backend: Backends) -> anyhow::Result<()> {
    let features = Some(format!(
        "{},{}",
        DEFAULT_FEATURES,
        backend.to_rtic_feature()
    ));

    command_parser(&CargoCommand::Doc { cargoarg, features }, false)?;
    Ok(())
}

/// Run cargo test on the selcted package or all packages
///
/// If no package is specified, loop through all packages
pub fn cargo_test(package: &PackageOpt, backend: Backends) -> anyhow::Result<()> {
    if let Some(package) = package.package {
        let cmd = TestMetadata::match_package(package, backend);
        command_parser(&cmd, false)?;
    } else {
        // Iterate over all workspace packages
        for package in [
            Package::Rtic,
            Package::RticCommon,
            Package::RticMacros,
            Package::RticMonotonics,
            Package::RticSync,
            Package::RticTime,
        ] {
            let mut error_messages = vec![];
            let cmd = &TestMetadata::match_package(package, backend);
            if let Err(err) = command_parser(&cmd, false) {
                error_messages.push(err);
            }

            if !error_messages.is_empty() {
                for err in error_messages {
                    error!("{err}");
                }
            }
        }
    }
    Ok(())
}

/// Use mdbook to build the book
pub fn cargo_book(cargoarg: &Option<&str>) -> anyhow::Result<()> {
    command_parser(
        &CargoCommand::Book {
            mdbookarg: cargoarg,
        },
        false,
    )?;
    Ok(())
}

/// Run examples
///
/// Supports updating the expected output via the overwrite argument
pub fn run_test(
    cargoarg: &Option<&str>,
    backend: Backends,
    examples: &[String],
    overwrite: bool,
) -> anyhow::Result<()> {
    examples.into_par_iter().for_each(|example| {
        let cmd = CargoCommand::ExampleBuild {
            cargoarg: &Some("--quiet"),
            example,
            target: backend.to_target(),
            features: Some(format!(
                "{},{}",
                DEFAULT_FEATURES,
                backend.to_rtic_feature()
            )),
            mode: BuildMode::Release,
        };
        if let Err(err) = command_parser(&cmd, false) {
            error!("{err}");
        }

        let cmd = CargoCommand::Qemu {
            cargoarg,
            example,
            target: backend.to_target(),
            features: Some(format!(
                "{},{}",
                DEFAULT_FEATURES,
                backend.to_rtic_feature()
            )),
            mode: BuildMode::Release,
        };

        if let Err(err) = command_parser(&cmd, overwrite) {
            error!("{err}");
        }
    });

    Ok(())
}

/// Check the binary sizes of examples
pub fn build_and_check_size(
    cargoarg: &Option<&str>,
    backend: Backends,
    examples: &[String],
    size_arguments: &Option<Sizearguments>,
) -> anyhow::Result<()> {
    examples.into_par_iter().for_each(|example| {
        // Make sure the requested example(s) are built
        let cmd = CargoCommand::ExampleBuild {
            cargoarg: &Some("--quiet"),
            example,
            target: backend.to_target(),
            features: Some(format!(
                "{},{}",
                DEFAULT_FEATURES,
                backend.to_rtic_feature()
            )),
            mode: BuildMode::Release,
        };
        if let Err(err) = command_parser(&cmd, false) {
            error!("{err}");
        }

        let cmd = CargoCommand::ExampleSize {
            cargoarg,
            example,
            target: backend.to_target(),
            features: Some(format!(
                "{},{}",
                DEFAULT_FEATURES,
                backend.to_rtic_feature()
            )),
            mode: BuildMode::Release,
            arguments: size_arguments.clone(),
        };
        if let Err(err) = command_parser(&cmd, false) {
            error!("{err}");
        }
    });

    Ok(())
}
