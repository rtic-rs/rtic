use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();

    if version_check::Channel::read().unwrap().is_nightly() {
        println!("cargo:rustc-cfg=rustc_is_nightly");
    }

    // These targets all have know support for the BASEPRI register.
    if target.starts_with("thumbv7m")
        | target.starts_with("thumbv7em")
        | target.starts_with("thumbv8m.main")
    {
        println!("cargo:rustc-cfg=have_basepri");

    // These targets are all known to _not_ have the BASEPRI register.
    } else if target.starts_with("thumb")
        && !(target.starts_with("thumbv6m") | target.starts_with("thumbv8m.base"))
    {
        panic!(
            "Unknown target '{}'. Need to update BASEPRI logic in build.rs.",
            target
        );
    }

    println!("cargo:rerun-if-changed=build.rs");
}
