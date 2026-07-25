fn main() {
    println!("cargo::rustc-link-arg=-Tlink.x");
    println!("cargo::rustc-link-arg=-Tdefmt.x");
    println!("cargo::rustc-link-arg=--nmagic");
}
