use crate::{
    analyze::Analysis as CodegenAnalysis,
    syntax::{analyze::Analysis as SyntaxAnalysis, ast::App},
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse, Attribute, Ident};

pub fn interrupt_ident() -> Ident {
    let span = Span::call_site();
    Ident::new("interrupt", span)
}

pub fn interrupt_mod(app: &App) -> TokenStream2 {
    let device = &app.args.device;
    let interrupt = interrupt_ident();
    quote!(#device::#interrupt)
}

pub fn impl_mutex(
    app: &App,
    analysis: &CodegenAnalysis,
    cfgs: &[Attribute],
    resources_prefix: bool,
    name: &Ident,
    ty: &TokenStream2,
    ceiling: u8,
    ptr: &TokenStream2,
) -> TokenStream2 {
    quote!()
}

pub fn extra_assertions(app: &App, analysis: &SyntaxAnalysis) -> Vec<TokenStream2> {
    vec![]
}

pub fn pre_init_preprocessing(app: &mut App, analysis: &SyntaxAnalysis) -> parse::Result<()> {
    Ok(())
}

pub fn pre_init_checks(app: &App, analysis: &SyntaxAnalysis) -> Vec<TokenStream2> {
    vec![]
}

pub fn pre_init_enable_interrupts(app: &App, analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    vec![]
}

pub fn architecture_specific_analysis(app: &App, analysis: &SyntaxAnalysis) -> parse::Result<()> {
    Ok(())
}

pub fn interrupt_entry(app: &App, analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    vec![]
}

pub fn interrupt_exit(app: &App, analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    vec![]
}

pub fn check_stack_overflow_before_init(
    _app: &App,
    _analysis: &CodegenAnalysis,
) -> Vec<TokenStream2> {
    vec![]
}

pub fn async_entry(
    app: &App,
    analysis: &CodegenAnalysis,
    dispatcher_name: Ident,
) -> Vec<TokenStream2> {
    vec![]
}

pub fn async_prio_limit(app: &App, analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    vec![]
}

pub fn handler_config(
    app: &App,
    analysis: &CodegenAnalysis,
    dispatcher_name: Ident,
) -> Vec<TokenStream2> {
    vec![]
}

pub fn extra_modules(app: &App, analysis: &SyntaxAnalysis) -> Vec<TokenStream2> {
    vec![]
}
