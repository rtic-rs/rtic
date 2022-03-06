#![doc(
    html_logo_url = "https://raw.githubusercontent.com/rtic-rs/cortex-m-rtic/master/book/en/src/RTIC.svg",
    html_favicon_url = "https://raw.githubusercontent.com/rtic-rs/cortex-m-rtic/master/book/en/src/RTIC.svg"
)]
//deny_warnings_placeholder_for_ci

extern crate proc_macro;

use proc_macro::TokenStream;
use std::{env, fs, path::Path};

use rtic_syntax::Settings;

mod analyze;
mod check;
mod codegen;
#[cfg(test)]
mod tests;

/// Attribute used to declare a RTIC application
///
/// For user documentation see the [RTIC book](https://rtic.rs)
///
/// # Panics
///
/// Should never panic, cargo feeds a path which is later converted to a string
#[proc_macro_attribute]
pub fn app(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut settings = Settings::default();
    settings.optimize_priorities = false;
    settings.parse_binds = true;
    settings.parse_extern_interrupt = true;

    let (app, analysis) = match rtic_syntax::parse(args, input, settings) {
        Err(e) => return e.to_compile_error().into(),
        Ok(x) => x,
    };

    let extra = match check::app(&app, &analysis) {
        Err(e) => return e.to_compile_error().into(),
        Ok(x) => x,
    };

    let analysis = analyze::app(analysis, &app);

    let ts = codegen::app(&app, &analysis, &extra);

    // Default output path: <project_dir>/target/
    let mut out_dir = Path::new("target");

    // Get output directory from Cargo environment
    // TODO don't want to break builds if OUT_DIR is not set, is this ever the case?
    let out_str = env::var("OUT_DIR").unwrap_or_else(|_| "".to_string());

    // Assuming we are building for a thumbv* target
    let target_triple_prefix = "thumbv";

    // Check for special scenario where default target/ directory is not present
    //
    // This is configurable in .cargo/config:
    //
    // [build]
    // target-dir = "target"
    #[cfg(feature = "debugprint")]
    println!("OUT_DIR\n{:#?}", out_str);

    if out_dir.exists() {
        #[cfg(feature = "debugprint")]
        println!("\ntarget/ exists\n");
    } else {
        // Set out_dir to OUT_DIR
        out_dir = Path::new(&out_str);

        // Default build path, annotated below:
        // $(pwd)/target/thumbv7em-none-eabihf/debug/build/cortex-m-rtic-<HASH>/out/
        // <project_dir>/<target-dir>/<TARGET>/debug/build/cortex-m-rtic-<HASH>/out/
        //
        // traverse up to first occurrence of TARGET, approximated with starts_with("thumbv")
        // and use the parent() of this path
        //
        // If no "target" directory is found, <project_dir>/<out_dir_root> is used
        for path in out_dir.ancestors() {
            if let Some(dir) = path.components().last() {
                if dir
                    .as_os_str()
                    .to_str()
                    .unwrap()
                    .starts_with(target_triple_prefix)
                {
                    if let Some(out) = path.parent() {
                        out_dir = out;
                        #[cfg(feature = "debugprint")]
                        println!("{:#?}\n", out_dir);
                        break;
                    }
                    // If no parent, just use it
                    out_dir = path;
                    break;
                }
            }
        }
    }

    // Try to write the expanded code to disk
    if let Some(out_str) = out_dir.to_str() {
        #[cfg(feature = "debugprint")]
        println!("Write file:\n{}/rtic-expansion.rs\n", out_str);
        fs::write(format!("{}/rtic-expansion.rs", out_str), ts.to_string()).ok();
    }

    ts.into()
}
