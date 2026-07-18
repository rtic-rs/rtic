fn non_default_features() -> impl Iterator<Item = String> {
    std::env::vars().filter_map(|(k, _)| {
        k.strip_prefix("CARGO_FEATURE_")
            .map(|v| v.to_lowercase().replace("_", "-"))
            .filter(|f| f != "default")
    })
}

fn main() {
    let mut features: Vec<_> = non_default_features().collect();

    let Some(feature) = features.pop() else {
        println!("cargo::error=No backend feature selected. Select a backend for `rtic` to resolve this problem.");
        return;
    };

    if !features.is_empty() {
        println!("cargo::error=More than one backend selected.");
        return;
    }

    println!("cargo::rustc-check-cfg=cfg(riscv_slic)");

    if feature == "riscv-clint" || feature == "riscv-mecall" {
        println!("cargo::rustc-cfg=riscv_slic");
    }

    println!("cargo::rerun-if-changed=build.rs");
}
