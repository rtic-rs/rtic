use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();

    if version_check::Channel::read().unwrap().is_nightly() {
        println!("cargo:rustc-cfg=rustc_is_nightly");
    }

    if target.starts_with("thumbv6m") {
        println!("cargo:rustc-cfg=armv6m");
    }

    if target.starts_with("thumbv7m")
        | target.starts_with("thumbv7em")
        | target.starts_with("thumbv8m")
    {
        println!("cargo:rustc-cfg=armv7m");
    }

    println!("cargo:rerun-if-changed=build.rs");
}
