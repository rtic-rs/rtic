use crate::codegen::bindings::pre_init_preprocessing;
use crate::syntax::{analyze::Analysis, ast::App};
use syn::parse;

pub fn app(app: &mut App, analysis: &Analysis) -> parse::Result<()> {
    pre_init_preprocessing(app, analysis)
}
