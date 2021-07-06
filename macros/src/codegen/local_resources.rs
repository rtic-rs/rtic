use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::ast::App;

use crate::{analyze::Analysis, check::Extra, codegen::util};

/// Generates `local` variables and local resource proxies
///
/// I.e. the `static` variables and theirs proxies.
pub fn codegen(
    app: &App,
    analysis: &Analysis,
    extra: &Extra,
) -> (
    // mod_app -- the `static` variables behind the proxies
    Vec<TokenStream2>,
    // mod_resources -- the `resources` module
    TokenStream2,
) {
    let mut mod_app = vec![];
    // let mut mod_resources: _ = vec![];

    // All local resources declared in the `#[local]' struct
    for (name, res) in &app.local_resources {
        // let expr = &res.expr; // TODO: Extract from tasks???...
        let cfgs = &res.cfgs;
        let ty = &res.ty;
        let mangled_name = util::mark_internal_ident(&util::static_local_resource_ident(name));

        let ty = quote!(rtic::RacyCell<core::mem::MaybeUninit<#ty>>);
        let expr = quote!(rtic::RacyCell::new(core::mem::MaybeUninit::uninit()));

        let attrs = &res.attrs;
        // late resources in `util::link_section_uninit`
        let section = util::link_section_uninit(true);

        // For future use
        // let doc = format!(" RTIC internal: {}:{}", file!(), line!());
        mod_app.push(quote!(
            #[allow(non_upper_case_globals)]
            // #[doc = #doc]
            #[doc(hidden)]
            #(#attrs)*
            #(#cfgs)*
            #section
            static #mangled_name: #ty = #expr;
        ));
    }

    // let mod_resources = if mod_resources.is_empty() {
    //     quote!()
    // } else {
    //     quote!(mod local_resources {
    //         #(#mod_resources)*
    //     })
    // };

    (mod_app, TokenStream2::new())
}
