#![deny(warnings)]

extern crate proc_macro;

use proc_macro::TokenStream;
use std::{fs, path::Path};

use rtfm_syntax::Settings;

mod analyze;
mod check;
mod codegen;
#[cfg(test)]
mod tests;

#[proc_macro_attribute]
pub fn app(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut settings = Settings::default();
    settings.optimize_priorities = true;
    settings.parse_binds = true;
    settings.parse_cores = cfg!(feature = "heterogeneous") || cfg!(feature = "homogeneous");
    settings.parse_extern_interrupt = true;
    settings.parse_schedule = true;

    let (app, analysis) = match rtfm_syntax::parse(args, input, settings) {
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
        fs::write("target/rtfm-expansion.rs", ts.to_string()).ok();
    }

    ts.into()
}
