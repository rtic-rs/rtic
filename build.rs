use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();

    if target.starts_with("thumbv6m") {
        println!("cargo:rustc-cfg=thumbv6m");
    }

    println!("cargo:rerun-if-changed=build.rs");
}
