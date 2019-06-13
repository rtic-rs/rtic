#![deny(warnings)]
#![recursion_limit = "128"]

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
    let (app, analysis) = match rtfm_syntax::parse(
        args,
        input,
        Settings {
            parse_cores: cfg!(feature = "heterogeneous"),
            parse_exception: true,
            parse_extern_interrupt: true,
            parse_interrupt: true,
            parse_schedule: true,
            optimize_priorities: true,
            ..Settings::default()
        },
    ) {
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
