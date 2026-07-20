use crate::syntax::ast::App;
use crate::{
    analyze::Analysis,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

/// Generates the `#[pre_rtic_hook]` function if needed
pub fn codegen(app: &App, _: &Analysis) -> TokenStream2 {
    if let Some(pre_rtic_hook) = &app.pre_rtic_hook {
        let attrs = &pre_rtic_hook.attrs;
        let name = &pre_rtic_hook.name;
        let stmts = &pre_rtic_hook.stmts;
        let user_idle = if !pre_rtic_hook.is_extern {
            Some(quote!(
                #(#attrs)*
                fn #name() {
                    #(#stmts)*
                }
            ))
        } else {
            None
        };

        quote!(
            #user_idle
        )
    } else {
        quote!()
    }
}
