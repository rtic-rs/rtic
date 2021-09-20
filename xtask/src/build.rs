use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{command::BuildMode, TestRunError};

const HEX_BUILD_ROOT: &str = "ci/builds";

/// make sure we're starting with a clean,but existing slate
pub fn init_build_dir() -> anyhow::Result<()> {
    if Path::new(HEX_BUILD_ROOT).exists() {
        fs::remove_dir_all(HEX_BUILD_ROOT)
            .map_err(|_| anyhow::anyhow!("Could not clear out directory:  {}", HEX_BUILD_ROOT))?;
    }
    fs::create_dir_all(HEX_BUILD_ROOT)
        .map_err(|_| anyhow::anyhow!("Could not create directory:  {}", HEX_BUILD_ROOT))
}

pub fn build_hexpath(
    example: &str,
    features: Option<&str>,
    build_mode: BuildMode,
    build_num: u32,
) -> anyhow::Result<String> {
    let features = match features {
        Some(f) => f,
        None => "",
    };

    let filename = format!("{}_{}_{}_{}.hex", example, features, build_mode, build_num);

    let mut path = PathBuf::from(HEX_BUILD_ROOT);
    path.push(filename);

    path.into_os_string()
        .into_string()
        .map_err(|e| anyhow::Error::new(TestRunError::PathConversionError(e)))
}

pub fn compare_builds(file_1: String, file_2: String) -> anyhow::Result<()> {
    let buf_1 = std::fs::read_to_string(file_1.clone())?;
    let buf_2 = std::fs::read_to_string(file_2.clone())?;

    if buf_1 != buf_2 {
        return Err(anyhow::Error::new(TestRunError::FileCmpError {
            file_1,
            file_2,
        }));
    }

    Ok(())
}
