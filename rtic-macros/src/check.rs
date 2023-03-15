use crate::codegen::bindings::architecture_specific_analysis;
use crate::syntax::{analyze::Analysis, ast::App};
use syn::parse;

pub fn app(app: &App, analysis: &Analysis) -> parse::Result<()> {
    architecture_specific_analysis(app, analysis)
}
