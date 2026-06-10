//! Emit a board-specific `memory.x` (flash/RAM sizes differ per chip) on the
//! linker search path.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Sizes follow the board feature selected in Cargo.toml.
    let (flash_k, ram_k) = if env::var_os("CARGO_FEATURE_XIAO_MG24").is_some() {
        (1536, 256) // EFR32MG24B220F1536IM48 (Seeed XIAO MG24)
    } else {
        (3200, 512) // EFR32MG26B420F3200IM68 (MGM260P Explorer Kit)
    };

    let memory_x = format!(
        "MEMORY\n{{\n  FLASH : ORIGIN = 0x08000000, LENGTH = {flash_k}K\n  \
         RAM   : ORIGIN = 0x20000000, LENGTH = {ram_k}K\n}}\n"
    );

    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(out.join("memory.x"), memory_x).unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_XIAO_MG24");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_MGM260P");
}
