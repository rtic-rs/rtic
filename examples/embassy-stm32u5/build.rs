fn main() {
    // Use cortex-m-rt's link.x as the linker script
    println!("cargo:rustc-link-arg=-Tlink.x");
    // Use defmt's linker script add-on
    println!("cargo:rustc-link-arg=-Tdefmt.x");
    // Force alignment
    println!("cargo:rustc-link-arg=-nmagic");
}
