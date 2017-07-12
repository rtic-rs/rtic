#![deny(warnings)]
#![feature(proc_macro)]
#![recursion_limit = "128"]

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

mod check;
mod syntax;
mod trans;
mod util;

use proc_macro::TokenStream;

#[proc_macro]
pub fn rtfm(ts: TokenStream) -> TokenStream {
    let input = format!("{}", ts);

    let app = syntax::parse::app(&input);
    let ceilings = util::compute_ceilings(&app);
    check::resources(&app.resources, &ceilings);

    format!("{}", trans::app(&app, &ceilings)).parse().unwrap()
}
