use crate::syntax::{ast::App, Context};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::codegen::util;

/// Generate shared resources structs
pub fn codegen(ctxt: Context, app: &App) -> (TokenStream2, TokenStream2) {
    let resources = match ctxt {
        Context::Init => unreachable!("Tried to generate shared resources struct for init"),
        Context::Idle => {
            &app.idle
                .as_ref()
                .expect("RTIC-ICE: unable to get idle name")
                .args
                .shared_resources
        }
        Context::HardwareTask(name) => &app.hardware_tasks[name].args.shared_resources,
        Context::SoftwareTask(name) => &app.software_tasks[name].args.shared_resources,
    };

    let mut fields = vec![];
    let mut values = vec![];

    for (name, access) in resources {
        let res = app.shared_resources.get(name).expect("UNREACHABLE");

        let cfgs = &res.cfgs;

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
                quote!('a)
            };

            fields.push(quote!(
                #(#cfgs)*
                pub #name: &#lt #mut_ #ty
            ));
        } else if access.is_shared() {
            fields.push(quote!(
                #(#cfgs)*
                pub #name: &'a #ty
            ));
        } else {
            fields.push(quote!(
                #(#cfgs)*
                pub #name: shared_resources::#shared_name<'a>
            ));

            values.push(quote!(
                #(#cfgs)*
                #name: shared_resources::#shared_name::new()

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

    fields.push(quote!(
        #[doc(hidden)]
        pub __rtic_internal_marker: core::marker::PhantomData<&'a ()>
    ));

    values.push(quote!(__rtic_internal_marker: core::marker::PhantomData));

    let doc = format!("Shared resources `{}` has access to", ctxt.ident(app));
    let ident = util::shared_resources_ident(ctxt, app);
    let item = quote!(
        #[allow(non_snake_case)]
        #[allow(non_camel_case_types)]
        #[doc = #doc]
        pub struct #ident<'a> {
            #(#fields,)*
        }
    );

    let constructor = quote!(
        impl<'a> #ident<'a> {
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
