fn non_default_features() -> impl Iterator<Item = String> {
    std::env::vars().filter_map(|(k, _)| {
        k.strip_prefix("CARGO_FEATURE_")
            .map(|v| v.to_lowercase().replace("_", "-"))
            .filter(|f| f != "default")
    })
}

fn main() {
    let features: Vec<_> = non_default_features().collect();

    if features.is_empty() {
        println!("cargo::error=No backend feature selected.");
        return;
    } else if features.len() > 1 {
        println!("cargo::error=More than one backend selected.");
        return;
    }

    println!("cargo::rerun-if-changed=build.rs");
}
