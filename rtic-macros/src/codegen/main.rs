use crate::{analyze::Analysis, codegen::util, syntax::ast::App};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use super::{assertions, post_init, pre_init};

/// Generates code for `fn main`
pub fn codegen(app: &App, analysis: &Analysis) -> TokenStream2 {
    let assertion_stmts = assertions::codegen(app, analysis);

    let pre_init_stmts = pre_init::codegen(app, analysis);

    let post_init_stmts = post_init::codegen(app, analysis);

    let call_idle = if let Some(idle) = &app.idle {
        let name = &idle.name;
        quote!(#name(#name::Context::new()))
    } else if analysis.channels.get(&0).is_some() {
        let dispatcher = util::zero_prio_dispatcher_ident();
        quote!(#dispatcher();)
    } else {
        quote!(loop {
            rtic::export::nop()
        })
    };

    let main = util::suffixed("main");
    let init_name = &app.init.name;
    quote!(
        #[doc(hidden)]
        #[no_mangle]
        unsafe extern "C" fn #main() -> ! {
            #(#assertion_stmts)*

            #(#pre_init_stmts)*

            #[inline(never)]
            fn __rtic_init_resources<F>(f: F) where F: FnOnce() {
                f();
            }

            // Wrap late_init_stmts in a function to ensure that stack space is reclaimed.
            __rtic_init_resources(||{
                let (shared_resources, local_resources) = #init_name(#init_name::Context::new(core.into()));

                #(#post_init_stmts)*
            });

            #call_idle
        }
    )
}
