use crate::{
    analyze::Analysis as CodegenAnalysis,
    syntax::{analyze::Analysis as SyntaxAnalysis, ast::App},
};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use std::cell::RefCell;
use syn::{parse, parse_str, Attribute, Ident, Path};

thread_local! {
    static PAC_PATH: RefCell<Option<String>> = RefCell::new(None);
}

pub fn interrupt_ident() -> Ident {
    let span = Span::call_site();
    Ident::new("Interrupt", span)
}

pub fn interrupt_mod(_app: &App) -> TokenStream2 {
    PAC_PATH.with(|p| {
        if let Some(s) = p.borrow().as_ref() {
            let pac: Path = parse_str(s).expect("stored pac path is valid");
            quote!(#pac::Interrupt)
        } else {
            quote!(esp32::Interrupt)
        }
    })
}

pub fn impl_mutex(
    _app: &App,
    _analysis: &CodegenAnalysis,
    _cfgs: &[Attribute],
    _resources_prefix: bool,
    _name: &Ident,
    _ty: &TokenStream2,
    _ceiling: u8,
    _ptr: &TokenStream2,
) -> TokenStream2 {
    quote!()
}

pub fn extra_assertions(_app: &App, _analysis: &SyntaxAnalysis) -> Vec<TokenStream2> {
    vec![]
}

pub fn pre_init_preprocessing(app: &mut App, _analysis: &SyntaxAnalysis) -> parse::Result<()> {
    let device = &app.args.device;
    let pac_str = quote!(#device).to_string();
    PAC_PATH.with(|p| *p.borrow_mut() = Some(pac_str));

    app.args.device = parse_str("crate :: __rtic_esp32_device")
        .expect("hardcoded path is valid");
    Ok(())
}

pub fn pre_init_checks(_app: &App, _analysis: &SyntaxAnalysis) -> Vec<TokenStream2> {
    vec![]
}

pub fn pre_init_enable_interrupts(_app: &App, _analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    vec![]
}

pub fn architecture_specific_analysis(_app: &App, _analysis: &SyntaxAnalysis) -> parse::Result<()> {
    Ok(())
}

pub fn interrupt_entry(_app: &App, _analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    vec![]
}

pub fn interrupt_exit(_app: &App, _analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    vec![]
}

pub fn check_stack_overflow_before_init(
    _app: &App,
    _analysis: &CodegenAnalysis,
) -> Vec<TokenStream2> {
    vec![]
}

pub fn async_entry(
    _app: &App,
    _analysis: &CodegenAnalysis,
    _dispatcher_name: Ident,
) -> Vec<TokenStream2> {
    vec![]
}

pub fn async_prio_limit(_app: &App, _analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    vec![]
}

pub fn handler_config(
    _app: &App,
    _analysis: &CodegenAnalysis,
    _dispatcher_name: Ident,
) -> Vec<TokenStream2> {
    vec![]
}

pub fn extra_modules(_app: &App, _analysis: &SyntaxAnalysis) -> Vec<TokenStream2> {
    vec![]
}

pub fn extra_top_level(_app: &App, _analysis: &SyntaxAnalysis) -> Vec<TokenStream2> {
    PAC_PATH.with(|p| {
        if let Some(s) = p.borrow().as_ref() {
            let pac: Path = parse_str(s).expect("stored pac path is valid");
            vec![quote!(
                mod __rtic_esp32_device {
                    pub use #pac::Interrupt;
                    pub type Peripherals = esp_hal::peripherals::Peripherals;
                }
            )]
        } else {
            vec![]
        }
    })
}
