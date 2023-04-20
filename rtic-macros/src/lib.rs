#![doc(
    html_logo_url = "https://raw.githubusercontent.com/rtic-rs/rtic/master/book/en/src/RTIC.svg",
    html_favicon_url = "https://raw.githubusercontent.com/rtic-rs/rtic/master/book/en/src/RTIC.svg"
)]

macro_rules! with_backend {
    (mod: [$($mod:tt),*]) => {
        $(
            with_backend!{ mod $mod; }
        )*
    };
    ($($tokens:tt)*) => {
        #[cfg(any(
            feature = "cortex-m-source-masking",
            feature = "cortex-m-basepri",
            feature = "test-template",
            feature = "riscv-esp32c3"
        ))]
        $($tokens)*
    };
}

with_backend! { mod: [analyze, check, codegen, syntax] }
with_backend! { use std::{fs, env, path::Path}; }
with_backend! { use proc_macro::TokenStream; }

with_backend! {
    // Used for mocking the API in testing
    #[doc(hidden)]
    #[proc_macro_attribute]
    pub fn mock_app(args: TokenStream, input: TokenStream) -> TokenStream {
        if let Err(e) = syntax::parse(args, input) {
            e.to_compile_error().into()
        } else {
            "fn main() {}".parse().unwrap()
        }
    }
}

with_backend! {
    /// Attribute used to declare a RTIC application
    ///
    /// For user documentation see the [RTIC book](https://rtic.rs)
    ///
    /// # Panics
    ///
    /// Should never panic, cargo feeds a path which is later converted to a string
    #[proc_macro_attribute]
    pub fn app(_args: TokenStream, _input: TokenStream) -> TokenStream {
        let (app, analysis) = match syntax::parse(_args, _input) {
            Err(e) => return e.to_compile_error().into(),
            Ok(x) => x,
        };

        if let Err(e) = check::app(&app, &analysis) {
            return e.to_compile_error().into();
        }

        let analysis = analyze::app(analysis, &app);

        let ts = codegen::app(&app, &analysis);

        // Default output path: <project_dir>/target/
        let mut out_dir = Path::new("target");

        // Get output directory from Cargo environment
        // TODO don't want to break builds if OUT_DIR is not set, is this ever the case?
        let out_str = env::var("OUT_DIR").unwrap_or_else(|_| "".to_string());

        if !out_dir.exists() {
            // Set out_dir to OUT_DIR
            out_dir = Path::new(&out_str);

            // Default build path, annotated below:
            // $(pwd)/target/thumbv7em-none-eabihf/debug/build/rtic-<HASH>/out/
            // <project_dir>/<target-dir>/<TARGET>/debug/build/rtic-<HASH>/out/
            //
            // traverse up to first occurrence of TARGET, approximated with starts_with("thumbv")
            // and use the parent() of this path
            //
            // If no "target" directory is found, <project_dir>/<out_dir_root> is used
            for path in out_dir.ancestors() {
                if let Some(dir) = path.components().last() {
                    let dir = dir.as_os_str().to_str().unwrap();

                    if dir.starts_with("thumbv") || dir.starts_with("riscv") {
                        if let Some(out) = path.parent() {
                            out_dir = out;
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
            fs::write(format!("{out_str}/rtic-expansion.rs"), ts.to_string()).ok();
        }

        ts.into()
    }
}

#[cfg(not(any(
    feature = "cortex-m-source-masking",
    feature = "cortex-m-basepri",
    feature = "test-template",
    feature = "riscv-esp32c3"
)))]
compile_error!("Cannot compile. No backend feature selected.");
