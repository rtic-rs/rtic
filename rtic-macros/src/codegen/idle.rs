use crate::syntax::{ast::App, Context};
use crate::{
    analyze::Analysis,
    codegen::{local_resources_struct, module, shared_resources_struct},
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

/// Generates support code for `#[idle]` functions
pub fn codegen(app: &App, analysis: &Analysis) -> TokenStream2 {
    if let Some(idle) = &app.idle {
        let mut mod_app = vec![];
        let mut root_idle = vec![];

        let name = &idle.name;

        if !idle.args.shared_resources.is_empty() {
            let (item, constructor) = shared_resources_struct::codegen(Context::Idle, app);

            root_idle.push(item);
            mod_app.push(constructor);
        }

        if !idle.args.local_resources.is_empty() {
            let (item, constructor) = local_resources_struct::codegen(Context::Idle, app);

            root_idle.push(item);

            mod_app.push(constructor);
        }

        root_idle.push(module::codegen(Context::Idle, app, analysis));

        let attrs = &idle.attrs;
        let context = &idle.context;
        let stmts = &idle.stmts;
        let user_idle = if !idle.is_extern {
            Some(quote!(
                #(#attrs)*
                #[allow(non_snake_case)]
                fn #name(#context: #name::Context) -> ! {
                    use rtic::Mutex as _;
                    use rtic::mutex::prelude::*;

                    #(#stmts)*
                }
            ))
        } else {
            None
        };

        quote!(
            #(#mod_app)*

            #(#root_idle)*

            #user_idle
        )
    } else {
        quote!()
    }
}
