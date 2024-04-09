use std::env;

fn main() {
    // Get the backend feature selected by the user
    let mut backends: Vec<_> = env::vars()
        .filter_map(|(key, _value)| {
            if key.starts_with("CARGO_FEATURE") && key.ends_with("BACKEND") {
                // strip 'CARGO_FEATURE_', convert to lowercase, and replace '_' with '-'
                Some(key[14..].to_lowercase().replace('_', "-"))
            } else {
                None
            }
        })
        .collect();
    if backends.len() > 1 {
        panic!("More than one backend feature selected: {:?}", backends);
    }
    let backend = backends.pop().expect("No backend feature selected.");

    match backend.as_str() {
        "thumbv6-backend" | "thumbv8base-backend" => {
            println!("cargo:rustc-cfg=feature=\"cortex-m-source-masking\"");
        }
        "thumbv7-backend" | "thumbv8main-backend" => {
            println!("cargo:rustc-cfg=feature=\"cortex-m-basepri\"");
        }
        "riscv-esp32c3-backend" => {
            println!("cargo:rustc-cfg=feature=\"riscv-esp32c3\"");
        }
        "riscv-esp32c6-backend" => {
            println!("cargo:rustc-cfg=feature=\"riscv-esp32c6\"");
        }
        "riscv-clint-backend" => {
            println!("cargo:rustc-cfg=feature=\"riscv-slic\"");
        }
        _ => {
            panic!("Unknown backend feature: {:?}", backend);
        }
    }

    println!("cargo:rerun-if-changed=build.rs");
}
