// #![deny(warnings)]

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/rtic-rs/cortex-m-rtic/master/book/en/src/RTIC.svg",
    html_favicon_url = "https://raw.githubusercontent.com/rtic-rs/cortex-m-rtic/master/book/en/src/RTIC.svg"
)]

extern crate proc_macro;

use proc_macro::TokenStream;
use std::{fs, path::Path};

use rtic_syntax::Settings;

mod analyze;
mod check;
mod codegen;
#[cfg(test)]
mod tests;

/// Attribute used to declare a RTIC application
///
/// For user documentation see the [RTIC book](https://rtic.rs)

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

    // Try to write the expanded code to disk
    if Path::new("target").exists() {
        fs::write("target/rtic-expansion.rs", ts.to_string()).ok();
    }

    ts.into()
}
