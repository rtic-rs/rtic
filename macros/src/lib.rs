#![feature(proc_macro)]
#![recursion_limit = "128"]

#[macro_use]
extern crate error_chain;
extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate rtfm_syntax as syntax;
extern crate syn;

use proc_macro::TokenStream;
use syntax::App;

use syntax::error::*;

mod analyze;
mod check;
mod trans;

#[proc_macro]
pub fn app(ts: TokenStream) -> TokenStream {
    match run(ts) {
        Err(e) => panic!("{}", error_chain::ChainedError::display(&e)),
        Ok(ts) => ts,
    }
}

fn run(ts: TokenStream) -> Result<TokenStream> {
    let input = format!("{}", ts);

    let app = App::parse(&input).chain_err(|| "parsing")?;
    let app = syntax::check::app(app).chain_err(|| "checking the AST")?;
    let app = check::app(app)?;

    let ownerships = analyze::app(&app);
    let tokens = trans::app(&app, &ownerships);

    Ok(format!("{}", tokens)
        .parse()
        .map_err(|_| "BUG: error parsing the generated code")?)
}
