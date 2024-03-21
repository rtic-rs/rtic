use super::bindings::extra_modules;
use crate::analyze::Analysis;
use crate::syntax::ast::App;
use proc_macro2::TokenStream as TokenStream2;

/// Generates code that runs before `#[init]`
pub fn codegen(app: &App, analysis: &Analysis) -> Vec<TokenStream2> {
    extra_modules(app, analysis)
}
