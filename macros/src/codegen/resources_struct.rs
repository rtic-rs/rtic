use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::{ast::App, Context};

use crate::codegen::util;

pub fn codegen(ctxt: Context, needs_lt: &mut bool, app: &App) -> (TokenStream2, TokenStream2) {
    let mut lt = None;

    let resources = match ctxt {
        Context::Init => &app.inits.first().unwrap().args.resources,
        Context::Idle => &app.idles.first().unwrap().args.resources,
        Context::HardwareTask(name) => &app.hardware_tasks[name].args.resources,
        Context::SoftwareTask(name) => &app.software_tasks[name].args.resources,
    };

    let mut fields = vec![];
    let mut values = vec![];
    let mut has_cfgs = false;

    for (name, access) in resources {
        let (res, expr) = app.resource(name).expect("UNREACHABLE");

        let cfgs = &res.cfgs;
        has_cfgs |= !cfgs.is_empty();

        // access hold if the resource is [x] (exclusive) or [&x] (shared)
        let mut_ = if access.is_exclusive() {
            Some(quote!(mut))
        } else {
            None
        };
        let ty = &res.ty;
        let mangled_name = util::mark_internal_ident(&name);

        // let ownership = &analysis.ownerships[name];
        let r_prop = &res.properties;

        if !r_prop.task_local && !r_prop.lock_free {
            if access.is_shared() {
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
                    pub #name: resources::#name<'a>
                ));

                values.push(quote!(
                    #(#cfgs)*
                    #name: resources::#name::new(priority)

                ));

                // continue as the value has been filled,
                continue;
            }
        } else {
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
        }

        let is_late = expr.is_none();
        if is_late {
            let expr = if access.is_exclusive() {
                quote!(&mut *#mangled_name.get_mut_unchecked().as_mut_ptr())
            } else {
                quote!(&*#mangled_name.as_ptr())
            };

            values.push(quote!(
                #(#cfgs)*
                #name: #expr
            ));
        } else {
            values.push(quote!(
                #(#cfgs)*
                #name: #mangled_name.get_mut_unchecked()
            ));
        }
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

    let doc = format!("Resources `{}` has access to", ctxt.ident(app));
    let ident = util::resources_ident(ctxt, app);
    let ident = util::mark_internal_ident(&ident);
    let item = quote!(
        #[allow(non_snake_case)]
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
