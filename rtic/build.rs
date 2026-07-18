use std::env;

fn backends() -> impl Iterator<Item = String> {
    env::vars().filter_map(|(k, _)| {
        k.strip_prefix("CARGO_FEATURE_").and_then(|f| {
            f.ends_with("BACKEND")
                .then_some(f.to_lowercase().replace("_", "-"))
        })
    })
}

fn main() {
    // Get the backend feature selected by the user
    let mut backends: Vec<_> = backends().collect();

    if backends.len() > 1 {
        println!("cargo::error=More than one backend selected: {backends:?}");
        return;
    }

    let Some(backend) = backends.pop() else {
        println!("cargo::error=No backend feature selected");
        return;
    };

    let features = [
        ("thumbv6-backend", "cortex-m-source-masking"),
        ("thumbv8base-backend", "cortex-m-source-masking"),
        ("thumbv7-backend", "cortex-m-basepri"),
        ("thumbv8main-backend", "cortex-m-basepri"),
        ("riscv-esp32c3-backend", "riscv-esp32c3"),
        ("riscv-esp32c6-backend", "riscv-esp32c6"),
        ("riscv-clint-backend", "riscv-slic"),
        ("riscv-mecall-backend", "riscv-slic"),
    ];

    let cfg_values: Vec<_> = features
        .iter()
        .map(|(_feature, cfg)| format!("\"{cfg}\""))
        .collect();

    let values = cfg_values.join(",");
    println!("cargo::rustc-check-cfg=cfg(feature, values({}))", values);

    if let Some(feature) = features
        .iter()
        .find_map(|(in_feature, out_feature)| (in_feature == &backend).then_some(out_feature))
    {
        println!("cargo::rustc-cfg=feature=\"{feature}\"");
    } else {
        println!("cargo::error=Unknown backend: {backend}");
        return;
    }

    println!("cargo::rerun-if-changed=build.rs");
}
