use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::{ast::App, Context};

use crate::codegen::util;

/// Generates local resources structs
pub fn codegen(ctxt: Context, needs_lt: &mut bool, app: &App) -> (TokenStream2, TokenStream2) {
    let mut lt = None;

    let resources = match ctxt {
        Context::Init => &app.init.args.local_resources,
        Context::Idle => &app.idle.unwrap().args.local_resources,
        Context::HardwareTask(name) => &app.hardware_tasks[name].args.local_resources,
        Context::SoftwareTask(name) => &app.software_tasks[name].args.local_resources,
    };

    let mut fields = vec![];
    let mut values = vec![];
    let mut has_cfgs = false;

    for (name, task_local) in resources {
        let res = app.local_resources.get(name).expect("UNREACHABLE");

        let cfgs = &res.cfgs;
        has_cfgs |= !cfgs.is_empty();

        let lt = if ctxt.runs_once() {
            quote!('static)
        } else {
            lt = Some(quote!('a));
            quote!('a)
        };

        let ty = &res.ty;
        let mangled_name = util::mark_internal_ident(&name);

        fields.push(quote!(
            #(#cfgs)*
            pub #name: &#lt mut #ty
        ));

        let expr = quote!(&mut *#mangled_name.get_mut_unchecked().as_mut_ptr());

        values.push(quote!(
            #(#cfgs)*
            #name: #expr
        ));
    }

    if lt.is_some() {
        *needs_lt = true;

        // The struct could end up empty due to `cfg`s leading to an error due to `'a` being unused
        if has_cfgs {
            fields.push(quote!(
                #[doc(hidden)]
                pub __marker__: core::marker::PhantomData<&'a ()>
            ));

            values.push(quote!(__marker__: core::marker::PhantomData))
        }
    }

    let doc = format!("Local resources `{}` has access to", ctxt.ident(app));
    let ident = util::local_resources_ident(ctxt, app);
    let ident = util::mark_internal_ident(&ident);
    let item = quote!(
        #[allow(non_snake_case)]
        #[doc = #doc]
        pub struct #ident<#lt> {
            #(#fields,)*
        }
    );

    let constructor = quote!(
        impl<#lt> #ident<#lt> {
            #[inline(always)]
            pub unsafe fn new() -> Self {
                #ident {
                    #(#values,)*
                }
            }
        }
    );

    (item, constructor)
}
