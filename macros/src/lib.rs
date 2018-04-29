// #![deny(warnings)]
#![allow(warnings)]
#![feature(proc_macro)]
#![recursion_limit = "256"]

#[macro_use]
extern crate failure;
extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate either;
extern crate rtfm_syntax as syntax;

use proc_macro::TokenStream;
use syntax::{App, Result};

mod analyze;
mod check;
mod trans;

#[proc_macro]
pub fn app(ts: TokenStream) -> TokenStream {
    match run(ts) {
        Err(e) => panic!("error: {}", e),
        Ok(ts) => ts,
    }
}

fn run(ts: TokenStream) -> Result<TokenStream> {
    let app = App::parse(ts)?.check()?;
    check::app(&app)?;

    let ctxt = analyze::app(&app);
    let tokens = trans::app(&ctxt, &app);

    Ok(tokens.into())
}
