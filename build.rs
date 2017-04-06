use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();

    if target == "thumbv6m-none-eabi" {
        println!("cargo:rustc-cfg=thumbv6m");
    }
}
