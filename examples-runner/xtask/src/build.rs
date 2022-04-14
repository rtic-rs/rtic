use std::{fs, path::Path};

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
