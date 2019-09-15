use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtfm_syntax::{
    ast::{App, Local},
    Context, Core, Map,
};

use crate::codegen::util;

pub fn codegen(
    ctxt: Context,
    locals: &Map<Local>,
    core: Core,
    app: &App,
) -> (
    // locals
    TokenStream2,
    // pat
    TokenStream2,
) {
    assert!(!locals.is_empty());

    let runs_once = ctxt.runs_once();
    let ident = util::locals_ident(ctxt, app);

    let mut lt = None;
    let mut fields = vec![];
    let mut items = vec![];
    let mut names = vec![];
    let mut values = vec![];
    let mut pats = vec![];
    let mut has_cfgs = false;

    for (name, local) in locals {
        let lt = if runs_once {
            quote!('static)
        } else {
            lt = Some(quote!('a));
            quote!('a)
        };

        let cfgs = &local.cfgs;
        has_cfgs |= !cfgs.is_empty();

        let section = if local.shared && cfg!(feature = "heterogeneous") {
            Some(quote!(#[rtfm::export::shared]))
        } else {
            util::link_section("data", core)
        };
        let expr = &local.expr;
        let ty = &local.ty;
        fields.push(quote!(
            #(#cfgs)*
            #name: &#lt mut #ty
        ));
        items.push(quote!(
            #(#cfgs)*
            #section
            static mut #name: #ty = #expr
        ));
        values.push(quote!(
            #(#cfgs)*
            #name: &mut #name
        ));
        names.push(name);
        pats.push(quote!(
            #(#cfgs)*
            #name
        ));
    }

    if lt.is_some() && has_cfgs {
        fields.push(quote!(__marker__: core::marker::PhantomData<&'a mut ()>));
        values.push(quote!(__marker__: core::marker::PhantomData));
    }

    let locals = quote!(
        #[allow(non_snake_case)]
        #[doc(hidden)]
        pub struct #ident<#lt> {
            #(#fields),*
        }

        impl<#lt> #ident<#lt> {
            #[inline(always)]
            unsafe fn new() -> Self {
                #(#items;)*

                #ident {
                    #(#values),*
                }
            }
        }
    );

    let ident = ctxt.ident(app);
    (
        locals,
        quote!(#ident::Locals { #(#pats,)* .. }: #ident::Locals),
    )
}
