use std::path::PathBuf;

use crate::{command::BuildMode, TestRunError};

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
    ["ci", "builds", &filename]
        .iter()
        .collect::<PathBuf>()
        .into_os_string()
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
