use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    // Put the linker script somewhere the linker can find it
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    #[cfg(feature = "embedded-ci")]
    let bytes = include_bytes!("embedded-ci.x");

    #[cfg(feature = "qemu")]
    let bytes = include_bytes!("qemu.x");

    // If running in test mode, use the memory layout that can be flashed
    // onto the chip directly
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(bytes)
        .unwrap();

    #[cfg(feature = "qemu")]
    File::create(out.join("defmt.x"))
        .unwrap()
        .write_all(b"")
        .unwrap();

    println!("cargo:rustc-link-search={}", out.display());

    // Only re-run the build script when memory.x is changed,
    // instead of when any part of the source code changes.
    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed=embedded-ci.x");
    println!("cargo:rerun-if-changed=qemu.x");
    println!("cargo:rerun-if-changed=link.x");
    println!("cargo:rerun-if-changed=build.rs");
}
