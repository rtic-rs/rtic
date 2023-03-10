use crate::{
    analyze::Analysis as CodegenAnalysis,
    syntax::{analyze::Analysis as SyntaxAnalysis, ast::App},
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse, Attribute, Ident};

const E310X_PLIC_PRIO_BITS: u8 = 3;

/// Implement `Mutex` using the PLIC threshold
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
    let path = if resources_prefix {
        quote!(shared_resources::#name)
    } else {
        quote!(#name)
    };

    let device = &app.args.device;
    quote!(
        #(#cfgs)*
        impl<'a> rtic::Mutex for #path<'a> {
            type T = #ty;

            #[inline(always)]
            fn lock<RTIC_INTERNAL_R>(&mut self, f: impl FnOnce(&mut #ty) -> RTIC_INTERNAL_R) -> RTIC_INTERNAL_R {
                const CEILING: u8 = #ceiling;

                unsafe {
                    rtic::export::lock(
                        #ptr,
                        CEILING,
                        /* FIXME: we need to work around this. The original
                        BASEPRI register has 8 bits to work with, a.k.a 255
                        priority levels. We only have 8 priority levels. */
                        E310X_PLIC_PRIO_BITS,
                        f,
                    )
                }
            }
        }
    )


    quote!() // TODO
}

pub fn extra_assertions(_app: &App, _analysis: &SyntaxAnalysis) -> Vec<TokenStream2> {
    vec![] // TODO
}

pub fn pre_init_checks(_app: &App, _analysis: &SyntaxAnalysis) -> Vec<TokenStream2> {
    vec![] // TODO
}

pub fn pre_init_enable_interrupts(_app: &App, _analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    vec![] // TODO
}

pub fn architecture_specific_analysis(_app: &App, _analysis: &SyntaxAnalysis) -> parse::Result<()> {
    Ok(()) // TODO
}

pub fn interrupt_entry(_app: &App, _analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    vec![] // TODO
}

pub fn interrupt_exit(_app: &App, _analysis: &CodegenAnalysis) -> Vec<TokenStream2> {
    vec![] // TODO
}
