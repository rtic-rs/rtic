use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::{ast::App, Context};

use crate::codegen::util;

/// Generate shared resources structs
pub fn codegen(ctxt: Context, needs_lt: &mut bool, app: &App) -> (TokenStream2, TokenStream2) {
    let mut lt = None;

    let resources = match ctxt {
        Context::Init => unreachable!("Tried to generate shared resources struct for init"),
        Context::Idle => &app.idle.as_ref().unwrap().args.shared_resources,
        Context::HardwareTask(name) => &app.hardware_tasks[name].args.shared_resources,
        Context::SoftwareTask(name) => &app.software_tasks[name].args.shared_resources,
    };

    let mut fields = vec![];
    let mut values = vec![];
    let mut has_cfgs = false;

    for (name, access) in resources {
        let res = app.shared_resources.get(name).expect("UNREACHABLE");

        let cfgs = &res.cfgs;
        has_cfgs |= !cfgs.is_empty();

        // access hold if the resource is [x] (exclusive) or [&x] (shared)
        let mut_ = if access.is_exclusive() {
            Some(quote!(mut))
        } else {
            None
        };
        let ty = &res.ty;
        let mangled_name = util::static_shared_resource_ident(name);
        let shared_name = util::need_to_lock_ident(name);

        if res.properties.lock_free {
            // Lock free resources of `idle` and `init` get 'static lifetime
            let lt = if ctxt.runs_once() {
                quote!('static)
            } else {
                lt = Some(quote!('a));
                quote!('a)
            };

            fields.push(quote!(
                #(#cfgs)*
                pub #name: &#lt #mut_ #ty
            ));
        } else if access.is_shared() {
            lt = Some(quote!('a));

            fields.push(quote!(
                #(#cfgs)*
                pub #name: &'a #ty
            ));
        } else {
            // Resource proxy
            lt = Some(quote!('a));

            fields.push(quote!(
                #(#cfgs)*
                pub #name: shared_resources::#shared_name<'a>
            ));

            values.push(quote!(
                #(#cfgs)*
                #name: shared_resources::#shared_name::new(priority)

            ));

            // continue as the value has been filled,
            continue;
        }

        let expr = if access.is_exclusive() {
            quote!(&mut *(&mut *#mangled_name.get_mut()).as_mut_ptr())
        } else {
            quote!(&*(&*#mangled_name.get()).as_ptr())
        };

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

            values.push(quote!(__marker__: core::marker::PhantomData));
        }
    }

    let doc = format!("Shared resources `{}` has access to", ctxt.ident(app));
    let ident = util::shared_resources_ident(ctxt, app);
    let item = quote!(
        #[allow(non_snake_case)]
        #[allow(non_camel_case_types)]
        #[doc = #doc]
        pub struct #ident<#lt> {
            #(#fields,)*
        }
    );

    let arg = if ctxt.is_init() {
        None
    } else {
        Some(quote!(priority: &#lt rtic::export::Priority))
    };
    let constructor = quote!(
        impl<#lt> #ident<#lt> {
            #[inline(always)]
            pub unsafe fn new(#arg) -> Self {
                #ident {
                    #(#values,)*
                }
            }
        }
    );

    (item, constructor)
}
