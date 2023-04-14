use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();

    // These targets all have know support for the BASEPRI register.
    if target.starts_with("thumbv7m")
        | target.starts_with("thumbv7em")
        | target.starts_with("thumbv8m.main")
    {
        println!("cargo:rustc-cfg=feature=\"cortex-m-basepri\"");
    } else if target.starts_with("thumbv6m") | target.starts_with("thumbv8m.base") {
        println!("cargo:rustc-cfg=feature=\"cortex-m-source-masking\"");
        //this should not be this general
        //riscv processors differ in interrupt implementation
        //even within the same target
        //need some other way to discern
    } else if target.starts_with("riscv32i") {
        println!("cargo:rustc-cfg=feature=\"riscv-esp32c3\"");

        // TODO: Add feature here for risc-v targets
        // println!("cargo:rustc-cfg=feature=\"riscv\"");
    } else if target.starts_with("thumb") || target.starts_with("riscv32") {
        panic!("Unknown target '{target}'. Need to update logic in build.rs.");
    }

    println!("cargo:rerun-if-changed=build.rs");
}
